use std::collections::{BTreeMap, BTreeSet};

use syn::punctuated::Punctuated;
use syn::{
    Attribute, Ident, Item, ItemExternCrate, ItemFn, ItemMacro, ItemTrait, ItemUse, Macro, Meta, Stmt, StmtMacro,
    Token, VisRestricted,
};

use code_model::{Building, CrateExports, Domain, Item as ModelItem, ItemId, ItemKind, ItemPath, Reexport, Unresolved};
use lint_core::Macros;
use lint_layout::{Casing, Dependencies, Name, Whole, Word, words};

use super::Namespace;
use super::PortMacro;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::{GlobId, ScopeId};
use super::resolver::{Resolution, Resolver};
use super::settling::{ScopeSettling, Seed, Target};
use super::std_names::{Expansion, StdNames};
use super::tables::{Explicit, GlobView, Key, Keyed, Layer, ModuleGlob, Row};
use super::use_path::{Leaf, UsePath};
use super::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
            DeclaredKind::Module(module) => Binding::Module(module),
            DeclaredKind::OpaqueModule => Binding::UNBOUND,
            DeclaredKind::Item(_) => Binding::Item(level, item),
        }
    }

    fn above(self, base: ScopeId) -> DeclaredKind {
        match self {
            DeclaredKind::Module(module) => DeclaredKind::Module(module.above(base)),
            DeclaredKind::OpaqueModule | DeclaredKind::Item(_) => self,
        }
    }

    fn item_kind(self) -> Option<ItemKind> {
        match self {
            DeclaredKind::Item(kind) => Some(kind),
            DeclaredKind::Module(_) | DeclaredKind::OpaqueModule => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Declared {
    name: Name,
    visibility: Visibility,
    kind: DeclaredKind,
    domain: Domain,
    marks: Vec<ItemPath>,
}

impl Declared {
    pub(crate) fn new(name: Name, visibility: Visibility, kind: DeclaredKind) -> Declared {
        Declared { name, visibility, kind, domain: Domain::Types, marks: Vec::new() }
    }

    fn named_item(ident: &Ident, visibility: &syn::Visibility, kind: ItemKind) -> Declared {
        Declared::new(Name::from(ident), Visibility::from(visibility), DeclaredKind::Item(kind))
    }

    fn valued(self) -> Declared {
        Declared { domain: Domain::Values, ..self }
    }

    fn marked(self, attributes: &[Attribute]) -> Declared {
        Declared {
            marks: attributes.iter().map(|attribute| UsePath::from(attribute.path()).written()).collect(),
            ..self
        }
    }

    fn issued(&self, building: &mut Building) -> Option<ItemId> {
        let kind = self.kind.item_kind()?;
        Some(building.declared(ModelItem::new(self.name.clone(), kind).marked(self.marks.clone())))
    }
}

#[derive(Debug)]
struct TraitEntries<'item>(&'item ItemTrait);

impl TraitEntries<'_> {
    fn entries(self) -> Entries {
        let trait_item = self.0;
        let filler = PortMacro::filler(trait_item).map(|name| Declared {
            name,
            visibility: Visibility::from(&trait_item.vis),
            kind: DeclaredKind::Item(ItemKind::Trait),
            domain: Domain::Types,
            marks: Vec::new(),
        });
        [Declared::named_item(&trait_item.ident, &trait_item.vis, ItemKind::Trait).marked(&trait_item.attrs)]
            .into_iter()
            .chain(filler)
            .map(Entries::from)
            .fold(Entries::default(), Entries::merged)
    }
}

#[derive(Clone, Copy, Debug)]
struct DeriveMacro<'item>(&'item ItemFn);

impl DeriveMacro<'_> {
    const ATTRIBUTE: [Word<'static>; 1] = words!["proc_macro_derive"];

    fn declared(self) -> Vec<Declared> {
        let visibility = Visibility::from(&self.0.vis);
        self.0
            .attrs
            .iter()
            .filter_map(DeriveMacro::named)
            .map(|name| Declared {
                name,
                visibility,
                kind: DeclaredKind::Item(ItemKind::Derive),
                domain: Domain::Macros,
                marks: Vec::new(),
            })
            .collect()
    }

    fn named(attribute: &Attribute) -> Option<Name> {
        attribute.path().get_ident().map(Name::from).and_then(|ident| ident.matches::<Whole>(&Self::ATTRIBUTE))?;
        let settings = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok()?;
        settings.first().and_then(|first| first.path().get_ident()).map(Name::from)
    }
}

#[derive(Debug)]
pub(crate) enum Imported {
    Named(Name, UsePath),
    Glob(UsePath),
}

impl Imported {
    const TYPED: [Domain; 2] = [Domain::Types, Domain::Macros];

    fn domains(name: &Name) -> &'static [Domain] {
        match name.casing() {
            Casing::Camel => &Self::TYPED,
            Casing::Lower | Casing::Shouting | Casing::Other => &Domain::ALL,
        }
    }

    fn entries(self, visibility: Visibility) -> Entries {
        match self {
            Imported::Named(name, path) => Self::domains(&name)
                .iter()
                .copied()
                .map(|domain| Entries::from(Import { name: name.clone(), path: path.clone(), visibility, domain }))
                .fold(Entries::default(), Entries::merged),
            Imported::Glob(path) => Entries::from(Glob { source: GlobSource::Path(path), visibility }),
        }
    }
}

#[derive(Debug)]
struct Import {
    name: Name,
    path: UsePath,
    visibility: Visibility,
    domain: Domain,
}

impl Import {
    fn row<'rows>(&'rows self, resolution: &'rows Resolution) -> Option<Keyed<'rows, Row<'rows>>> {
        let row = Row::new(Layer::Imported, self.visibility, View::from(resolution.present()?));
        Some(Keyed::new(Key::new(self.domain, &self.name), row))
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
            _ => space.global(&self.krate, space.dependencies(scope)),
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

    fn unresolved(&self) -> Unresolved {
        match self {
            GlobSource::Path(_) => Unresolved::Unbound,
            GlobSource::Expansion => Unresolved::PossiblyMacroDeclared,
        }
    }

    fn reexport(&self, dependencies: &Dependencies, declared: &BTreeSet<&Name>) -> Reexport {
        match self {
            GlobSource::Path(path) => path.reexport(dependencies, declared),
            GlobSource::Expansion => Reexport::Opaque,
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
            Item::Macro(ItemMacro { ident: None, mac, .. }) => {
                Entries::from(mac).merged(Entries::of_items(&Macros::items(mac)))
            }
            Item::Macro(ItemMacro { ident: Some(ident), .. }) => Entries::from(Declared {
                domain: Domain::Macros,
                ..Declared::named_item(ident, &syn::Visibility::Inherited, ItemKind::Macro)
            }),
            Item::Struct(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Struct))
            }
            Item::Enum(declared) => Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Enum)),
            Item::Union(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Union))
            }
            Item::Type(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Alias))
            }
            Item::Trait(declared) => TraitEntries(declared).entries(),
            Item::TraitAlias(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Trait))
            }
            Item::Fn(declared) => DeriveMacro(declared)
                .declared()
                .into_iter()
                .chain([Declared::named_item(&declared.sig.ident, &declared.vis, ItemKind::Function).valued()])
                .map(Entries::from)
                .fold(Entries::default(), Entries::merged),
            Item::Const(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Const).valued())
            }
            Item::Static(declared) => {
                Entries::from(Declared::named_item(&declared.ident, &declared.vis, ItemKind::Static).valued())
            }
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

    pub(super) fn above(self, base: ScopeId) -> Entries {
        let declared = self
            .declared
            .into_iter()
            .map(|declared| Declared { kind: declared.kind.above(base), ..declared })
            .collect();
        Entries { declared, ..self }
    }

    pub(super) fn issued(&self, building: &mut Building) -> Vec<Option<ItemId>> {
        self.declared.iter().map(|declared| declared.issued(building)).collect()
    }

    pub(super) fn exported(&self) -> impl Iterator<Item = Key<'_>> {
        let declared =
            self.declared.iter().map(|declared| (Key::new(declared.domain, &declared.name), declared.visibility));
        let imported = self.imports.iter().map(|import| (Key::new(import.domain, &import.name), import.visibility));
        declared.chain(imported).filter(|(_, visibility)| *visibility == Visibility::Visible).map(|(key, _)| key)
    }

    pub(super) fn unresolved(&self) -> Unresolved {
        self.globs.iter().map(|glob| glob.source.unresolved()).max().unwrap_or(Unresolved::Unbound)
    }

    pub(super) fn prelude(&self, name: &Name, space: &Namespace, root: ScopeId) -> Option<Binding> {
        self.externs.get(name).and_then(|externs| externs.first()).map(|external| external.bound(space, root))
    }

    pub(super) fn seeds(&self, scope: ScopeId, space: &Namespace) -> Vec<Seed<'_>> {
        let declared = self.declared.iter().enumerate().map(|(index, declared)| {
            Seed::new(
                Key::new(declared.domain, &declared.name),
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
            self.imports
                .iter()
                .map(|import| {
                    resolver.importing(Key::new(import.domain, &import.name)).path(&import.path, scope, import.domain)
                })
                .collect(),
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
        let imported = self.imports.iter().zip(imports).filter_map(|(import, resolution)| import.row(resolution));
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

    pub(crate) fn crate_exports(&self, dependencies: &Dependencies, modules: &[Name]) -> CrateExports {
        let shown = |visibility: Visibility| visibility == Visibility::Visible;
        let declared: BTreeSet<&Name> = self
            .declared
            .iter()
            .filter(|declared| shown(declared.visibility))
            .filter(|declared| declared.domain != Domain::Values)
            .map(|declared| &declared.name)
            .chain(modules)
            .collect();
        let imported = self
            .imports
            .iter()
            .filter(|import| shown(import.visibility))
            .map(|import| (import.name.clone(), import.path.reexport(dependencies, &declared)));
        let externs = self.externs.iter().flat_map(|(name, externs)| {
            externs
                .iter()
                .filter(|external| shown(external.visibility))
                .map(|external| (name.clone(), UsePath::global(&external.krate).reexport(dependencies, &declared)))
        });
        let globs = self
            .globs
            .iter()
            .filter(|glob| shown(glob.visibility))
            .map(|glob| glob.source.reexport(dependencies, &declared));
        let with_declared =
            declared.iter().fold(CrateExports::default(), |exports, name| exports.with_declared((*name).clone()));
        let with_named =
            imported.chain(externs).fold(with_declared, |exports, (name, reexport)| exports.with_named(name, reexport));
        globs.fold(with_named, CrateExports::with_glob)
    }

    fn merged(mut self, other: Entries) -> Entries {
        self.declared.extend(other.declared);
        self.imports.extend(other.imports);
        other.externs.into_iter().for_each(|(name, externs)| self.externs.entry(name).or_default().extend(externs));
        self.globs.extend(other.globs);
        self
    }
}
