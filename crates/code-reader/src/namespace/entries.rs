use syn::{Ident, Item, ItemExternCrate, ItemMacro, ItemUse, Stmt, VisRestricted};

use code_model::ItemPath;
use lint_layout::Name;

use super::anchor::Anchor;
use super::binding::{Binding, Globbed};
use super::boundary::ScopeId;
use super::resolver::Walk;
use super::use_path::{Leaf, UsePath};

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
    fn seen(self, via: Via) -> Admission {
        match self {
            Visibility::Private => Admission::Hidden,
            Visibility::Visible => Admission::Shown,
            Visibility::Restricted => via.restricted(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Via {
    Path,
    Glob,
}

impl Via {
    fn restricted(self) -> Admission {
        match self {
            Via::Path => Admission::Shown,
            Via::Glob => Admission::Obscured,
        }
    }

    pub(super) fn outside(self) -> Sight {
        match self {
            Via::Path => Sight::ByPath,
            Via::Glob => Sight::ByGlob,
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
    fn admits(self, visibility: Visibility) -> Admission {
        match self {
            Sight::Inside => Admission::Shown,
            Sight::ByPath => visibility.seen(Via::Path),
            Sight::ByGlob => visibility.seen(Via::Glob),
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

#[derive(Clone, Copy, Debug)]
enum Tally {
    Zero,
    One(DeclaredKind, ItemId),
    Many,
    Blurred,
}

impl Tally {
    fn admitted(self, admission: Admission, kind: DeclaredKind, item: ItemId) -> Tally {
        match admission {
            Admission::Hidden => self,
            Admission::Shown => self.shown(kind, item),
            Admission::Obscured => Tally::Blurred,
        }
    }

    fn shown(self, kind: DeclaredKind, item: ItemId) -> Tally {
        match self {
            Tally::Zero => Tally::One(kind, item),
            Tally::One(..) | Tally::Many => Tally::Many,
            Tally::Blurred => Tally::Blurred,
        }
    }

    fn globbed(self, level: ScopeId) -> Globbed {
        match self {
            Tally::Zero => Globbed::Absent,
            Tally::One(kind, item) => Globbed::Present(kind.bound(level, item)),
            Tally::Many => Globbed::Present(Binding::Unknown),
            Tally::Blurred => Globbed::Opaque,
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

#[derive(Debug)]
struct Extern {
    name: Name,
    visibility: Visibility,
    krate: Name,
}

impl Extern {
    fn binding(declared: &ItemExternCrate) -> Option<Extern> {
        declared
            .rename
            .iter()
            .map(|(_, renamed)| renamed)
            .chain([&declared.ident])
            .next()
            .map(Name::from)
            .and_then(|name| Leaf::from(name).standalone())
            .map(|name| Extern {
                name,
                visibility: Visibility::from(&declared.vis),
                krate: Name::from(&declared.ident),
            })
    }

    fn found(externs: &[Extern], name: &Name, sight: Sight, walk: &Walk<'_>) -> Globbed {
        externs
            .iter()
            .filter(|external| external.name == *name)
            .map(|external| {
                Globbed::admitted(sight.admits(external.visibility), || Globbed::Present(external.bound(walk)))
            })
            .fold(Globbed::Absent, Globbed::joined)
    }

    fn bound(&self, walk: &Walk<'_>) -> Binding {
        match Anchor::spelled(&self.krate) {
            Some(Anchor::Module) => Binding::from(Anchor::Crate.anchored(walk.space().anchors(walk.scope()))),
            _ => walk.space().global(&self.krate),
        }
    }
}

#[derive(Debug)]
enum GlobSource {
    Path(UsePath),
    Expansion,
}

impl GlobSource {
    fn globbed(&self, name: &Name, walk: &Walk<'_>) -> Globbed {
        match self {
            GlobSource::Path(path) => walk.expanding(path).globbed(walk, path, name),
            GlobSource::Expansion => Globbed::Opaque,
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
struct Declarations(Vec<Declared>);

impl Declarations {
    fn found(&self, level: ScopeId, name: &Name, sight: Sight) -> Globbed {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, declared)| declared.name == *name)
            .fold(Tally::Zero, |tally, (index, declared)| {
                tally.admitted(sight.admits(declared.visibility), declared.kind, ItemId(index))
            })
            .globbed(level)
    }
}

#[derive(Debug, Default)]
struct Imports(Vec<Import>);

impl Imports {
    fn found(&self, name: &Name, sight: Sight, walk: &Walk<'_>) -> Globbed {
        self.0
            .iter()
            .filter(|import| import.name == *name)
            .map(|import| {
                Globbed::admitted(sight.admits(import.visibility), || Globbed::Present(walk.path(&import.path)))
            })
            .fold(Globbed::Absent, Globbed::joined)
    }
}

#[derive(Debug, Default)]
struct Globs(Vec<Glob>);

impl Globs {
    fn found(&self, name: &Name, sight: Sight, walk: &Walk<'_>) -> Globbed {
        self.0
            .iter()
            .map(|glob| Globbed::admitted(sight.admits(glob.visibility), || glob.source.globbed(name, walk)))
            .fold(Globbed::Absent, Globbed::joined)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Entries {
    declared: Declarations,
    imports: Imports,
    externs: Vec<Extern>,
    globs: Globs,
}

impl From<Declared> for Entries {
    fn from(declared: Declared) -> Entries {
        Entries { declared: Declarations(vec![declared]), ..Entries::default() }
    }
}

impl From<Import> for Entries {
    fn from(import: Import) -> Entries {
        Entries { imports: Imports(vec![import]), ..Entries::default() }
    }
}

impl From<Glob> for Entries {
    fn from(glob: Glob) -> Entries {
        Entries { globs: Globs(vec![glob]), ..Entries::default() }
    }
}

impl From<Option<Extern>> for Entries {
    fn from(external: Option<Extern>) -> Entries {
        Entries { externs: external.into_iter().collect(), ..Entries::default() }
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
            Item::ExternCrate(declared) => Entries::from(Extern::binding(declared)),
            Item::Macro(ItemMacro { ident: None, .. }) => Entries::from(Glob::EXPANSION),
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

impl Entries {
    pub(crate) fn of_items(items: &[Item]) -> Entries {
        items.iter().map(Entries::from).fold(Entries::default(), Entries::merged)
    }

    pub(crate) fn of_block(statements: &[Stmt]) -> Entries {
        statements.iter().map(Entries::of_statement).fold(Entries::default(), Entries::merged)
    }

    pub(super) fn declare(&mut self, declared: Declared) {
        self.declared.0.push(declared);
    }

    pub(super) fn found(&self, level: ScopeId, name: &Name, walk: &Walk<'_>) -> Globbed {
        let sight = walk.view(level);
        let at = walk.at_level(level);
        let domain = walk.domain();
        domain
            .typed(|| self.declared.found(level, name, sight))
            .or_else(|| self.imports.found(name, sight, &at))
            .or_else(|| domain.typed(|| Extern::found(&self.externs, name, sight, &at)))
            .or_else(|| self.globs.found(name, sight, &at))
    }

    pub(super) fn path_of(&self, item: ItemId, located: Option<ItemPath>) -> Option<ItemPath> {
        let name = self.declared.0.get(item.0).map(|declared| &declared.name);
        located.zip(name).map(|(path, name)| path.joined(name))
    }

    pub(super) fn within(&self, module: ScopeId, located: Option<ItemPath>) -> Option<ItemPath> {
        let name = self
            .declared
            .0
            .iter()
            .find(|declared| declared.kind == DeclaredKind::Module(module))
            .map(|declared| &declared.name);
        located.zip(name).map(|(path, name)| path.joined(name))
    }

    pub(super) fn prelude(&self, name: &Name, walk: &Walk<'_>) -> Option<Binding> {
        self.externs.iter().find(|external| external.name == *name).map(|external| external.bound(walk))
    }

    fn of_statement(statement: &Stmt) -> Entries {
        match statement {
            Stmt::Item(item) => Entries::from(item),
            Stmt::Macro(_) => Entries::from(Glob::EXPANSION),
            Stmt::Local(_) | Stmt::Expr(..) => Entries::default(),
        }
    }

    fn merged(mut self, other: Entries) -> Entries {
        self.declared.0.extend(other.declared.0);
        self.imports.0.extend(other.imports.0);
        self.externs.extend(other.externs);
        self.globs.0.extend(other.globs.0);
        self
    }
}
