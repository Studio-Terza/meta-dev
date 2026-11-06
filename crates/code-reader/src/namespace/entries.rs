use std::collections::BTreeMap;

use syn::{Ident, Item, ItemExternCrate, ItemMacro, ItemUse, Macro, Stmt, StmtMacro, VisRestricted};

use code_model::ItemPath;
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
    fn from(visibility: &syn::Visibility) -> Visibility {
        match visibility {
            syn::Visibility::Inherited => Visibility::Private,
            syn::Visibility::Public(_) => Visibility::Visible,
            syn::Visibility::Restricted(restricted) => Visibility::from(restricted),
        }
    }
}

impl From<&VisRestricted> for Visibility {
    fn from(restricted: &VisRestricted) -> Visibility {
        restricted
            .in_token
            .map(|_| Visibility::Restricted)
            .or_else(|| {
                restricted.path.get_ident().map(Name::from).as_ref().and_then(Anchor::spelled).map(Visibility::from)
            })
            .unwrap_or(Visibility::Restricted)
    }
}

impl Visibility {
    fn seen(self, restricted: Admission) -> Admission {
        match self {
            Visibility::Private => Admission::Hidden,
            Visibility::Visible => Admission::Shown,
            Visibility::Restricted => restricted,
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
            Sight::Inside => Admission::Shown,
            Sight::ByPath => visibility.seen(Admission::Shown),
            Sight::ByGlob => visibility.seen(Admission::Obscured),
        }
    }

    pub(super) fn beyond(self, reach: Option<ScopeId>) -> Sight {
        match reach {
            Some(_) => Sight::Inside,
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
pub(crate) struct ItemId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredKind {
    Module(ScopeId),
    OpaqueModule,
    Item,
}

impl DeclaredKind {
    fn bound(self, level: ScopeId, item: ItemId) -> Binding {
        match self {
            DeclaredKind::Module(module) => Binding::Module(module),
            DeclaredKind::OpaqueModule => Binding::Unknown,
            DeclaredKind::Item => Binding::Item(level, item),
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
    pub(crate) fn new(name: Name, visibility: Visibility, kind: DeclaredKind) -> Declared {
        Declared { name, visibility, kind }
    }

    fn named_item(ident: &Ident, visibility: &syn::Visibility) -> Declared {
        Declared { name: Name::from(ident), visibility: Visibility::from(visibility), kind: DeclaredKind::Item }
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
            Imported::Named(name, path) => Entries::from(Import { name, path, visibility }),
            Imported::Glob(path) => Entries::from(Glob { source: GlobSource::Path(path), visibility }),
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
    fn keyed(declared: &ItemExternCrate) -> BTreeMap<Name, Vec<Extern>> {
        declared
            .rename
            .iter()
            .map(|(_, renamed)| renamed)
            .chain([&declared.ident])
            .next()
            .map(Name::from)
            .and_then(|name| Leaf::from(name).standalone())
            .map(|name| {
                (name, vec![Extern { visibility: Visibility::from(&declared.vis), krate: Name::from(&declared.ident) }])
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
            GlobSource::Path(_) => Target::Waiting,
            GlobSource::Expansion => Target::Opaque,
        }
    }

    fn target(&self, resolver: &Resolver<'_, '_>, scope: ScopeId, glob: GlobId) -> Target {
        match self {
            GlobSource::Path(path) => resolver.glob_target(path, scope, glob),
            GlobSource::Expansion => Target::Opaque,
        }
    }
}

#[derive(Debug)]
struct Glob {
    source: GlobSource,
    visibility: Visibility,
}

impl Glob {
    const EXPANSION: Glob = Glob { source: GlobSource::Expansion, visibility: Visibility::Visible };
}

#[derive(Debug, Default)]
pub(crate) struct Entries {
    declared: Vec<Declared>,
    imports: Vec<Import>,
    externs: BTreeMap<Name, Vec<Extern>>,
    globs: Vec<Glob>,
}

impl From<Declared> for Entries {
    fn from(declared: Declared) -> Entries {
        Entries { declared: vec![declared], imports: Vec::new(), externs: BTreeMap::new(), globs: Vec::new() }
    }
}

impl From<Import> for Entries {
    fn from(import: Import) -> Entries {
        Entries { declared: Vec::new(), imports: vec![import], externs: BTreeMap::new(), globs: Vec::new() }
    }
}

impl From<Glob> for Entries {
    fn from(glob: Glob) -> Entries {
        Entries { declared: Vec::new(), imports: Vec::new(), externs: BTreeMap::new(), globs: vec![glob] }
    }
}

impl From<&ItemExternCrate> for Entries {
    fn from(declared: &ItemExternCrate) -> Entries {
        Entries { declared: Vec::new(), imports: Vec::new(), externs: Extern::keyed(declared), globs: Vec::new() }
    }
}

impl From<&Macro> for Entries {
    fn from(invoked: &Macro) -> Entries {
        match StdNames::expansion(invoked) {
            Expansion::Statement => Entries::default(),
            Expansion::Items => Entries::from(Glob::EXPANSION),
        }
    }
}

impl From<&ItemUse> for Entries {
    fn from(used: &ItemUse) -> Entries {
        let visibility = Visibility::from(&used.vis);
        UsePath::imported(used)
            .into_iter()
            .map(|imported| imported.entries(visibility))
            .fold(Entries::default(), Entries::merged)
    }
}

impl From<&Item> for Entries {
    fn from(item: &Item) -> Entries {
        match item {
            Item::Use(used) => Entries::from(used),
            Item::ExternCrate(declared) => Entries::from(declared),
            Item::Macro(ItemMacro { ident: None, mac, .. }) => Entries::from(mac),
            Item::Struct(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Enum(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Union(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Type(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Trait(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            Item::TraitAlias(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis)),
            _ => Entries::default(),
        }
    }
}

impl From<&Stmt> for Entries {
    fn from(statement: &Stmt) -> Entries {
        match statement {
            Stmt::Item(item) => Entries::from(item),
            Stmt::Macro(StmtMacro { mac, .. }) => Entries::from(mac),
            Stmt::Local(_) | Stmt::Expr(..) => Entries::default(),
        }
    }
}

impl Entries {
    pub(crate) fn of_items(items: &[Item]) -> Entries {
        items.iter().map(Entries::from).fold(Entries::default(), Entries::merged)
    }

    pub(crate) fn of_block(statements: &[Stmt]) -> Entries {
        statements.iter().map(Entries::from).fold(Entries::default(), Entries::merged)
    }

    pub(super) fn declare(&mut self, declared: Declared) {
        self.declared.push(declared);
    }

    pub(super) fn path_of(&self, item: ItemId, located: Option<ItemPath>) -> Option<ItemPath> {
        let name = self.declared.get(item.0).map(|declared| &declared.name);
        located.zip(name).map(|(path, name)| path.joined(name))
    }

    pub(super) fn within(&self, module: ScopeId, located: Option<ItemPath>) -> Option<ItemPath> {
        let name = self
            .declared
            .iter()
            .find(|declared| declared.kind == DeclaredKind::Module(module))
            .map(|declared| &declared.name);
        located.zip(name).map(|(path, name)| path.joined(name))
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
                declared.kind.bound(scope, ItemId(index)),
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

    fn merged(mut self, other: Entries) -> Entries {
        self.declared.extend(other.declared);
        self.imports.extend(other.imports);
        other.externs.into_iter().for_each(|(name, externs)| self.externs.entry(name).or_default().extend(externs));
        self.globs.extend(other.globs);
        self
    }
}
