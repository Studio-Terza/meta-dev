use std::collections::BTreeMap;

use syn::{Ident, Item, ItemExternCrate, ItemMacro, ItemUse, Macro, Stmt, StmtMacro, VisRestricted};

use code_model::{Building, Item as ModelItem, ItemId, ItemKind};
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::{GlobId, ScopeId};
use super::resolver::{Resolution, Resolver};
use super::settling::{ScopeSettling, Seed, Target};
use super::std_names::{Expansion, StdNames};
use super::tables::{Domain, Explicit, GlobView, Key, Keyed, Layer, ModuleGlob, Row};
use super::use_path::{Leaf, UsePath};
use super::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    Private,
    Visible,
    Restricted,
}

impl From<&syn::Visibility> for Visibility {
    fn from(visibility: &syn::Visibility) -> Self {
        match visibility {
            syn::Visibility::Inherited => Self::Private,
            syn::Visibility::Public(_) => Self::Visible,
            syn::Visibility::Restricted(restricted) => Self::from(restricted),
        }
    }
}

impl From<&VisRestricted> for Visibility {
    fn from(restricted: &VisRestricted) -> Self {
        restricted
            .in_token
            .map(|_| Self::Restricted)
            .or_else(|| restricted.path.get_ident().map(Name::from).as_ref().and_then(Anchor::spelled).map(Self::from))
            .unwrap_or(Self::Restricted)
    }
}

impl Visibility {
    fn seen(self, restricted: Admission) -> Admission {
        match self {
            Self::Private => Admission::Hidden,
            Self::Visible => Admission::Shown,
            Self::Restricted => restricted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sight {
    Inside,
    ByPath,
    ByGlob,
}

impl Sight {
    pub(super) fn admits(self, visibility: Visibility) -> Admission {
        match self {
            Self::Inside => Admission::Shown,
            Self::ByPath => visibility.seen(Admission::Shown),
            Self::ByGlob => visibility.seen(Admission::Obscured),
        }
    }

    pub(super) fn beyond(self, reach: Option<ScopeId>) -> Self {
        match reach {
            Some(_) => Self::Inside,
            None => self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Admission {
    Hidden,
    Shown,
    Obscured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeclaredId(usize);

impl DeclaredId {
    pub(super) fn row<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredKind {
    Module(ScopeId),
    OpaqueModule,
    Item(ItemKind),
}

impl DeclaredKind {
    fn bound(self, level: ScopeId, item: DeclaredId) -> Binding {
        match self {
            Self::Module(module) => Binding::Module(module),
            Self::OpaqueModule => Binding::Unknown,
            Self::Item(_) => Binding::Item(level, item),
        }
    }

    fn item_kind(self) -> Option<ItemKind> {
        match self {
            Self::Item(kind) => Some(kind),
            Self::Module(_) | Self::OpaqueModule => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Declared {
    name: Name,
    visibility: Visibility,
    kind: DeclaredKind,
}

impl Declared {
    pub(crate) fn new(name: Name, visibility: Visibility, kind: DeclaredKind) -> Self {
        Self { name, visibility, kind }
    }

    fn named_item(ident: &Ident, visibility: &syn::Visibility, kind: ItemKind) -> Self {
        Self { name: Name::from(ident), visibility: Visibility::from(visibility), kind: DeclaredKind::Item(kind) }
    }

    fn issued(&self, building: &mut Building) -> Option<ItemId> {
        let kind = self.kind.item_kind()?;
        Some(building.declared(ModelItem::new(self.name.clone(), kind)))
    }
}

#[derive(Debug)]
pub(crate) enum Imported {
    Named(Name, UsePath),
    Glob(UsePath),
}

impl Imported {
    fn entries(self, visibility: Visibility) -> Entries {
        match self {
            Self::Named(name, path) => Entries::from(Import { name, path, visibility }),
            Self::Glob(path) => Entries::from(Glob { source: GlobSource::Path(path), visibility }),
        }
    }
}

#[derive(Debug)]
struct Import {
    name: Name,
    path: UsePath,
    visibility: Visibility,
}

impl Import {
    fn rows<'rows>(&'rows self, resolution: &'rows Resolution) -> impl Iterator<Item = Keyed<'rows, Row<'rows>>> {
        let row = Row::new(Layer::Imported, self.visibility, View::from(resolution));
        Key::every(&self.name).map(move |key| Keyed::new(key, row))
    }
}

#[derive(Debug)]
struct Extern {
    visibility: Visibility,
    krate: Name,
}

impl Extern {
    fn keyed(declared: &ItemExternCrate) -> BTreeMap<Name, Vec<Self>> {
        declared
            .rename
            .iter()
            .map(|(_, renamed)| renamed)
            .chain([&declared.ident])
            .next()
            .map(Name::from)
            .and_then(|name| Leaf::from(name).standalone())
            .map(|name| {
                (name, vec![Self { visibility: Visibility::from(&declared.vis), krate: Name::from(&declared.ident) }])
            })
            .into_iter()
            .collect()
    }

    fn bound(&self, space: &Namespace, scope: ScopeId) -> Binding {
        match Anchor::spelled(&self.krate) {
            Some(Anchor::Module) => Binding::from(Anchor::Crate.anchored(space.anchors(scope))),
            _ => space.global(&self.krate),
        }
    }
}

#[derive(Debug)]
enum GlobSource {
    Path(UsePath),
    Expansion,
}

impl GlobSource {
    fn unsettled(&self) -> Target {
        match self {
            Self::Path(_) => Target::Waiting,
            Self::Expansion => Target::Opaque,
        }
    }

    fn target(&self, resolver: &Resolver<'_, '_>, scope: ScopeId, glob: GlobId) -> Target {
        match self {
            Self::Path(path) => resolver.glob_target(path, scope, glob),
            Self::Expansion => Target::Opaque,
        }
    }
}

#[derive(Debug)]
struct Glob {
    source: GlobSource,
    visibility: Visibility,
}

impl Glob {
    const EXPANSION: Self = Self { source: GlobSource::Expansion, visibility: Visibility::Visible };
}

#[derive(Debug, Default)]
pub(crate) struct Entries {
    declared: Vec<Declared>,
    imports: Vec<Import>,
    externs: BTreeMap<Name, Vec<Extern>>,
    globs: Vec<Glob>,
}

impl From<Declared> for Entries {
    fn from(declared: Declared) -> Self {
        Self { declared: vec![declared], imports: Vec::new(), externs: BTreeMap::new(), globs: Vec::new() }
    }
}

impl From<Import> for Entries {
    fn from(import: Import) -> Self {
        Self { declared: Vec::new(), imports: vec![import], externs: BTreeMap::new(), globs: Vec::new() }
    }
}

impl From<Glob> for Entries {
    fn from(glob: Glob) -> Self {
        Self { declared: Vec::new(), imports: Vec::new(), externs: BTreeMap::new(), globs: vec![glob] }
    }
}

impl From<&ItemExternCrate> for Entries {
    fn from(declared: &ItemExternCrate) -> Self {
        Self { declared: Vec::new(), imports: Vec::new(), externs: Extern::keyed(declared), globs: Vec::new() }
    }
}

impl From<&Macro> for Entries {
    fn from(invoked: &Macro) -> Self {
        match StdNames::expansion(invoked) {
            Expansion::Statement => Self::default(),
            Expansion::Items => Self::from(Glob::EXPANSION),
        }
    }
}

impl From<&ItemUse> for Entries {
    fn from(used: &ItemUse) -> Self {
        let visibility = Visibility::from(&used.vis);
        UsePath::imported(used)
            .into_iter()
            .map(|imported| imported.entries(visibility))
            .fold(Self::default(), Self::merged)
    }
}

impl From<&Item> for Entries {
    fn from(item: &Item) -> Self {
        match item {
            Item::Use(used) => Self::from(used),
            Item::ExternCrate(declared) => Self::from(declared),
            Item::Macro(ItemMacro { ident: None, mac, .. }) => Self::from(mac),
            Item::Struct(declared) => {
                Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Struct))
            }
            Item::Enum(declared) => Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Enum)),
            Item::Union(declared) => Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Union)),
            Item::Type(declared) => Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Alias)),
            Item::Trait(declared) => Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Trait)),
            Item::TraitAlias(declared) => {
                Self::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Trait))
            }
            _ => Self::default(),
        }
    }
}

impl From<&Stmt> for Entries {
    fn from(statement: &Stmt) -> Self {
        match statement {
            Stmt::Item(item) => Self::from(item),
            Stmt::Macro(StmtMacro { mac, .. }) => Self::from(mac),
            Stmt::Local(_) | Stmt::Expr(..) => Self::default(),
        }
    }
}

impl Entries {
    pub(crate) fn of_items(items: &[Item]) -> Self {
        items.iter().map(Self::from).fold(Self::default(), Self::merged)
    }

    pub(crate) fn of_block(statements: &[Stmt]) -> Self {
        statements.iter().map(Self::from).fold(Self::default(), Self::merged)
    }

    pub(super) fn declare(&mut self, declared: Declared) {
        self.declared.push(declared);
    }

    pub(super) fn issued(&self, building: &mut Building) -> Vec<Option<ItemId>> {
        self.declared.iter().map(|declared| declared.issued(building)).collect()
    }

    pub(super) fn prelude(&self, name: &Name, space: &Namespace, root: ScopeId) -> Option<Binding> {
        self.externs.get(name).and_then(|externs| externs.first()).map(|external| external.bound(space, root))
    }

    pub(super) fn seeds(&self, scope: ScopeId, space: &Namespace) -> Vec<Seed<'_>> {
        let declared = self.declared.iter().enumerate().map(|(index, declared)| {
            Seed::new(
                Key::new(Domain::Types, &declared.name),
                Layer::Declared,
                declared.visibility,
                declared.kind.bound(scope, DeclaredId(index)),
            )
        });
        let externs = self.externs.iter().flat_map(|(name, externs)| {
            externs.iter().map(move |external| {
                Seed::new(
                    Key::new(Domain::Types, name),
                    Layer::Extern,
                    external.visibility,
                    external.bound(space, scope),
                )
            })
        });
        declared.chain(externs).collect()
    }

    pub(super) fn unsettled(&self) -> ScopeSettling {
        ScopeSettling::new(
            self.imports.iter().map(|_| Resolution::Waiting).collect(),
            self.globs.iter().map(|glob| glob.source.unsettled()).collect(),
        )
    }

    pub(super) fn settled(&self, scope: ScopeId, resolver: &Resolver<'_, '_>) -> ScopeSettling {
        ScopeSettling::new(
            self.imports.iter().map(|import| resolver.path(&import.path, scope)).collect(),
            self.globs
                .iter()
                .zip(GlobId::ids(&self.globs))
                .map(|(glob, id)| glob.source.target(resolver, scope, id))
                .collect(),
        )
    }

    pub(super) fn explicit<'rows>(
        &'rows self,
        seeds: &'rows [Seed<'rows>],
        imports: &'rows [Resolution],
    ) -> Explicit<'rows> {
        let imported = self.imports.iter().zip(imports).flat_map(|(import, resolution)| import.rows(resolution));
        seeds.iter().map(Seed::row).chain(imported).collect()
    }

    pub(super) fn fixed<'rows>(&self, targets: &'rows [Target]) -> Vec<GlobView<'rows>> {
        let globs = self.globs.iter().zip(GlobId::ids(&self.globs)).zip(targets);
        globs.filter_map(|((glob, id), target)| target.fixed(id, glob.visibility)).collect()
    }

    pub(super) fn modules<'rows>(&self, targets: &'rows [Target]) -> Vec<ModuleGlob<'rows>> {
        let globs = self.globs.iter().zip(GlobId::ids(&self.globs)).zip(targets);
        globs.filter_map(|((glob, id), target)| target.module(id, glob.visibility)).collect()
    }

    fn merged(mut self, other: Self) -> Self {
        self.declared.extend(other.declared);
        self.imports.extend(other.imports);
        other.externs.into_iter().for_each(|(name, externs)| self.externs.entry(name).or_default().extend(externs));
        self.globs.extend(other.globs);
        self
    }
}
