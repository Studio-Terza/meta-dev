use std::collections::{BTreeMap, BTreeSet};

use algebra::{Monoid, Semigroup};
use syntax::MacroTree;
use syntax::Punctuated;
use syntax::{
    Attribute, File, Ident, Item, ItemExternCrate, ItemFn, ItemMacro, ItemTrait, ItemUse, Macro, Meta, Stmt, StmtMacro,
    Token, VisRestricted,
};

use graph::{CrateExports, Domain, Exposure, Item as ModelItem, ItemKind, ItemPath, Reexport, Unplaced};
use lint_layout::{Casing, Dependencies, Name, Whole, Word, words};

use super::Namespace;
use super::PortMacro;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::{GlobId, ScopeId};
use super::resolver::{Resolution, Resolver};
use super::rules::{RulesDefinition, Written};
use super::settling::{ScopeSettling, Seed, Target};
use super::std_names::{Expansion, StdNames};
use super::tables::{Declaring, Explicit, GlobView, Key, Keyed, Layer, ModuleGlob, Row};
use super::use_path::{Leaf, UsePath};
use super::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Visibility {
    Private,
    Visible,
    Restricted,
}

impl From<&syntax::Visibility> for Visibility {
    fn from(visibility: &syntax::Visibility) -> Visibility {
        match visibility {
            syntax::Visibility::Inherited => Visibility::Private,
            syntax::Visibility::Public(_) => Visibility::Visible,
            syntax::Visibility::Restricted(restricted) => Visibility::from(restricted),
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
    fn settled(self) -> Option<Admission> {
        match self {
            Visibility::Private => Some(Admission::Hidden),
            Visibility::Visible => Some(Admission::Shown),
            Visibility::Restricted => None,
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
            Sight::ByPath => visibility.settled().unwrap_or(Admission::Shown),
            Sight::ByGlob => visibility.settled().unwrap_or(Admission::Obscured),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeclaredAt {
    scope: ScopeId,
    item: DeclaredId,
}

impl DeclaredAt {
    pub(super) fn new(scope: ScopeId, item: DeclaredId) -> DeclaredAt {
        DeclaredAt { scope, item }
    }

    pub(super) fn scope(self) -> ScopeId {
        self.scope
    }

    pub(super) fn declared_id(self) -> DeclaredId {
        self.item
    }
}

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
    Rules(Export),
    Alias(DeclaredAt),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Export {
    Local,
    Exported,
}

impl DeclaredKind {
    const EXPORTED: DeclaredKind = DeclaredKind::Rules(Export::Exported);

    fn rules(self) -> Option<RulesDefinition> {
        match self {
            DeclaredKind::Rules(_) => Some(RulesDefinition),
            DeclaredKind::Module(_) | DeclaredKind::OpaqueModule | DeclaredKind::Item(_) | DeclaredKind::Alias(_) => {
                None
            }
        }
    }

    fn bound(self, level: ScopeId, item: DeclaredId) -> Binding {
        match self {
            DeclaredKind::Module(module) => Binding::Module(module),
            DeclaredKind::OpaqueModule => Binding::UNBOUND,
            DeclaredKind::Item(_) | DeclaredKind::Rules(_) => Binding::Item(level, item),
            DeclaredKind::Alias(at) => Binding::Item(at.scope(), at.declared_id()),
        }
    }

    pub(super) fn module(self) -> Option<ScopeId> {
        match self {
            DeclaredKind::Module(module) => Some(module),
            DeclaredKind::OpaqueModule | DeclaredKind::Item(_) | DeclaredKind::Rules(_) | DeclaredKind::Alias(_) => {
                None
            }
        }
    }

    fn item_kind(self) -> Option<ItemKind> {
        match self {
            DeclaredKind::Item(kind) => Some(kind),
            DeclaredKind::Rules(_) => Some(ItemKind::Macro),
            DeclaredKind::Module(_) | DeclaredKind::OpaqueModule | DeclaredKind::Alias(_) => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Below {
    scope: ScopeId,
    modules: Vec<Name>,
}

impl Below {
    pub(super) fn root(scope: ScopeId, krate: &Name) -> Below {
        Below { scope, modules: vec![krate.clone()] }
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn modules(&self) -> &[Name] {
        &self.modules
    }

    pub(super) fn deeper(&self, entries: &Entries) -> Vec<Below> {
        entries
            .declared
            .iter()
            .filter(|declared| declared.visibility == Visibility::Visible)
            .filter_map(|declared| {
                Some(Below {
                    scope: declared.kind.module()?,
                    modules: self.modules.iter().chain([&declared.name]).cloned().collect(),
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Declared {
    name: Name,
    visibility: Visibility,
    kind: DeclaredKind,
    domain: Domain,
    marks: Vec<ItemPath>,
    exposure: Exposure,
}

impl Declared {
    pub(crate) fn new(name: Name, visibility: Visibility, kind: DeclaredKind) -> Declared {
        Declared { name, visibility, kind, domain: Domain::Types, marks: Vec::new(), exposure: Exposure::Hidden }
    }

    fn named_item(ident: &Ident, visibility: &syntax::Visibility, kind: ItemKind) -> Declared {
        let exposure = Exposing::declared_by(visibility);
        Declared {
            exposure,
            ..Declared::new(Name::from(ident), Visibility::from(visibility), DeclaredKind::Item(kind))
        }
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

    fn issued(&self, krate: Option<&Name>) -> Option<ModelItem> {
        let kind = self.kind.item_kind()?;
        Some(Unplaced::new(self.name.clone(), kind).placed(self.marks.clone(), krate.cloned(), self.exposure))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exposing;

impl Exposing {
    pub(crate) fn declared_by(visibility: &syntax::Visibility) -> Exposure {
        match visibility {
            syntax::Visibility::Public(_) => Exposure::Public,
            syntax::Visibility::Restricted(_) => Exposure::Restricted,
            syntax::Visibility::Inherited => Exposure::Hidden,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct MacroRules<'item> {
    ident: &'item Ident,
    attributes: &'item [Attribute],
}

impl MacroRules<'_> {
    const EXPORT: [Word<'static>; 1] = words!["macro_export"];

    fn entries(self) -> Entries {
        Entries::from(Declared {
            domain: Domain::Macros,
            ..Declared::new(Name::from(self.ident), Visibility::Private, DeclaredKind::Rules(self.export()))
        })
    }

    fn export(self) -> Export {
        match self.attributes.iter().find_map(MacroRules::export_mark) {
            Some(()) => Export::Exported,
            None => Export::Local,
        }
    }

    fn export_mark(attribute: &Attribute) -> Option<()> {
        attribute.path().get_ident().map(Name::from).and_then(|name| name.matches::<Whole>(&Self::EXPORT)).map(|_| ())
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
            exposure: Exposing::declared_by(&trait_item.vis),
        });
        [Declared::named_item(&trait_item.ident, &trait_item.vis, ItemKind::Trait).marked(&trait_item.attrs)]
            .into_iter()
            .chain(filler)
            .map(Entries::from)
            .collect::<Entries>()
    }
}

#[derive(Clone, Copy, Debug)]
struct DeriveMacro<'item>(&'item ItemFn);

impl DeriveMacro<'_> {
    const ATTRIBUTE: [Word<'static>; 1] = words!["proc_macro_derive"];
    const FUNCTION_LIKE: [Word<'static>; 2] = words!["proc_macro", "proc_macro_attribute"];

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
                exposure: Exposing::declared_by(&self.0.vis),
            })
            .chain(self.function_like())
            .collect()
    }

    fn function_like(self) -> Option<Declared> {
        let marked = self.0.attrs.iter().find_map(|attribute| {
            attribute.path().get_ident().map(Name::from).and_then(|ident| ident.matches::<Whole>(&Self::FUNCTION_LIKE))
        });
        marked.map(|_| Declared {
            name: Name::from(&self.0.sig.ident),
            visibility: Visibility::from(&self.0.vis),
            kind: DeclaredKind::Item(ItemKind::Macro),
            domain: Domain::Macros,
            marks: Vec::new(),
            exposure: Exposing::declared_by(&self.0.vis),
        })
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
                .collect::<Entries>(),
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
        let row = Row::new(Layer::Imported, self.visibility, Option::<View>::from(resolution)?);
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
    Invoked(UsePath, Written),
}

impl GlobSource {
    fn unsettled(&self) -> Target {
        match self {
            GlobSource::Path(_) | GlobSource::Invoked(..) => Target::Waiting,
        }
    }

    fn declaring<'rows>(&self, target: &'rows Target) -> Declaring<'rows> {
        match self {
            GlobSource::Path(_) => Declaring::Nothing,
            GlobSource::Invoked(..) => target.declaring(),
        }
    }

    fn reexport(&self, dependencies: &Dependencies, declared: &BTreeSet<&Name>) -> Reexport {
        match self {
            GlobSource::Path(path) => path.reexport(dependencies, declared),
            GlobSource::Invoked(..) => Reexport::Opaque,
        }
    }

    fn target(&self, resolver: &Resolver<'_, '_>, scope: ScopeId, glob: GlobId) -> Target {
        match self {
            GlobSource::Path(path) => resolver.glob_target(path, scope, glob),
            GlobSource::Invoked(path, written) => resolver.invoked_target(path, scope, glob, written),
        }
    }
}

#[derive(Debug)]
struct Glob {
    source: GlobSource,
    visibility: Visibility,
}

impl Glob {
    fn invoked(invoked: &Macro) -> Glob {
        Glob {
            source: GlobSource::Invoked(UsePath::from(&invoked.path), Written::from(invoked)),
            visibility: Visibility::Visible,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Local(Entries);

impl From<Entries> for Local {
    fn from(entries: Entries) -> Local {
        Local(entries)
    }
}

impl Local {
    pub(super) fn placed(self, base: ScopeId) -> Entries {
        let rebased = |declared: Declared| {
            let kind = declared.kind.module().map_or(declared.kind, |module| DeclaredKind::Module(module.above(base)));
            Declared { kind, ..declared }
        };
        Entries { declared: self.0.declared.into_iter().map(rebased).collect(), ..self.0 }
    }
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
            Expansion::Items => Entries::from(Glob::invoked(invoked)),
        }
    }
}

impl From<&ItemUse> for Entries {
    fn from(used: &ItemUse) -> Entries {
        let visibility = Visibility::from(&used.vis);
        UsePath::imported(used).into_iter().map(|imported| imported.entries(visibility)).collect::<Entries>()
    }
}

impl From<&Item> for Entries {
    fn from(item: &Item) -> Entries {
        match item {
            Item::Use(used) => Entries::from(used),
            Item::ExternCrate(declared) => Entries::from(declared),
            Item::Macro(ItemMacro { ident: None, mac, .. }) => Entries::from(mac),
            Item::Macro(ItemMacro { ident: Some(ident), attrs, .. }) => {
                MacroRules { ident, attributes: attrs }.entries()
            }
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
                .collect::<Entries>(),
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
        items.iter().map(Entries::from).collect::<Entries>()
    }

    pub(crate) fn of_root(file: &File) -> Entries {
        [Entries::of_items(&file.items), Entries::of_items(&MacroTree::expanded_items(&file.items))]
            .into_iter()
            .collect()
    }

    pub(crate) fn of_block(statements: &[Stmt]) -> Entries {
        statements.iter().map(Entries::from).collect::<Entries>()
    }

    pub(super) fn of_declared(declared: Declared) -> Entries {
        Entries { declared: vec![declared], ..Entries::default() }
    }

    pub(super) fn exported_aliases(&self, scope: ScopeId) -> Vec<Declared> {
        let exported = self.declared.iter().enumerate().filter(|(_, declared)| declared.kind == DeclaredKind::EXPORTED);
        let aliased =
            exported.map(|(index, declared)| (declared.name.clone(), DeclaredAt::new(scope, DeclaredId(index))));
        aliased
            .map(|(name, at)| Declared {
                domain: Domain::Macros,
                ..Declared::new(name, Visibility::Visible, DeclaredKind::Alias(at))
            })
            .collect()
    }

    pub(super) fn issued(&self, krate: Option<&Name>) -> Vec<Option<ModelItem>> {
        self.declared.iter().map(|declared| declared.issued(krate)).collect()
    }

    pub(super) fn exported(&self) -> impl Iterator<Item = Key<'_>> {
        let declared =
            self.declared.iter().map(|declared| (Key::new(declared.domain, &declared.name), declared.visibility));
        let imported = self.imports.iter().map(|import| (Key::new(import.domain, &import.name), import.visibility));
        declared.chain(imported).filter(|(_, visibility)| *visibility == Visibility::Visible).map(|(key, _)| key)
    }

    pub(super) fn rules(&self, item: DeclaredId) -> Option<RulesDefinition> {
        item.row(&self.declared).and_then(|declared| declared.kind.rules())
    }

    pub(super) fn declaring<'rows>(&self, targets: &'rows [Target]) -> Vec<Declaring<'rows>> {
        self.globs.iter().zip(targets).map(|(glob, target)| glob.source.declaring(target)).collect()
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
                    resolver.importer(Key::new(import.domain, &import.name)).path(&import.path, scope, import.domain)
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
        let own: CrateExports = declared.iter().map(|name| (*name).clone()).collect();
        let named: CrateExports = imported.chain(externs).collect();
        own.combine(named).combine(globs.collect())
    }
}

impl Semigroup for Entries {
    fn combine(self, other: Entries) -> Entries {
        let mut externs = self.externs;
        other.externs.into_iter().for_each(|(name, found)| externs.entry(name).or_default().extend(found));
        Entries {
            declared: self.declared.combine(other.declared),
            imports: self.imports.combine(other.imports),
            externs,
            globs: self.globs.combine(other.globs),
        }
    }
}

impl Monoid for Entries {}

impl FromIterator<Entries> for Entries {
    fn from_iter<Parts: IntoIterator<Item = Entries>>(parts: Parts) -> Entries {
        Monoid::concat(parts)
    }
}
