use syn::{Ident, Item, ItemMacro, ItemUse, Stmt};

use lint_layout::Name;

use super::binding::{Binding, Globbed};
use super::boundary::ScopeId;
use super::resolver::Walk;
use super::use_path::UsePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    Private,
    Visible,
}

impl From<&syn::Visibility> for Visibility {
    fn from(visibility: &syn::Visibility) -> Visibility {
        match visibility {
            syn::Visibility::Inherited => Visibility::Private,
            syn::Visibility::Public(_) | syn::Visibility::Restricted(_) => Visibility::Visible,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reach {
    Inside,
    Outside,
}

impl Reach {
    const EVERY: [Visibility; 2] = [Visibility::Private, Visibility::Visible];
    const PUBLIC: [Visibility; 1] = [Visibility::Visible];

    pub(super) fn seen(self) -> &'static [Visibility] {
        match self {
            Reach::Inside => &Self::EVERY,
            Reach::Outside => &Self::PUBLIC,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredKind {
    Module(ScopeId),
    OpaqueModule,
    Item,
}

impl DeclaredKind {
    fn bound(self, level: ScopeId, name: &Name) -> Binding {
        match self {
            DeclaredKind::Module(module) => Binding::Module(module),
            DeclaredKind::OpaqueModule => Binding::Unknown,
            DeclaredKind::Item => Binding::Item(level, name.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Tally {
    Zero,
    One(DeclaredKind),
    Many,
}

impl Tally {
    fn counted(self, kind: DeclaredKind) -> Tally {
        match self {
            Tally::Zero => Tally::One(kind),
            Tally::One(_) | Tally::Many => Tally::Many,
        }
    }

    fn globbed(self, level: ScopeId, name: &Name) -> Globbed {
        match self {
            Tally::Zero => Globbed::Absent,
            Tally::One(kind) => Globbed::Present(kind.bound(level, name)),
            Tally::Many => Globbed::Present(Binding::Unknown),
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

    fn of_item(item: &Item) -> Option<Declared> {
        match item {
            Item::Struct(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Enum(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Union(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Type(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            Item::Trait(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            Item::TraitAlias(declared) => Some(Declared::named_item(&declared.ident, &declared.vis)),
            _ => None,
        }
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
    fn import(&self, visibility: Visibility) -> Option<Import> {
        match self {
            Imported::Named(name, path) => Some(Import { name: name.clone(), path: path.clone(), visibility }),
            Imported::Glob(_) => None,
        }
    }

    fn glob(&self, visibility: Visibility) -> Option<Glob> {
        match self {
            Imported::Named(..) => None,
            Imported::Glob(path) => Some(Glob { source: GlobSource::Path(path.clone()), visibility }),
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

    fn of_item(item: &Item) -> Option<Glob> {
        match item {
            Item::Macro(ItemMacro { ident: None, .. }) => Some(Glob::EXPANSION),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct Declarations(Vec<Declared>);

impl Declarations {
    fn found(&self, level: ScopeId, name: &Name, seen: &[Visibility]) -> Globbed {
        self.0
            .iter()
            .filter(|declared| declared.name == *name)
            .filter(|declared| seen.contains(&declared.visibility))
            .fold(Tally::Zero, |tally, declared| tally.counted(declared.kind))
            .globbed(level, name)
    }
}

#[derive(Debug, Default)]
struct Imports(Vec<Import>);

impl Imports {
    fn found(&self, name: &Name, seen: &[Visibility], walk: &Walk<'_>) -> Globbed {
        self.0
            .iter()
            .filter(|import| import.name == *name)
            .filter(|import| seen.contains(&import.visibility))
            .map(|import| Globbed::Present(walk.path(&import.path)))
            .fold(Globbed::Absent, Globbed::joined)
    }
}

#[derive(Debug, Default)]
struct Globs(Vec<Glob>);

impl Globs {
    fn found(&self, name: &Name, seen: &[Visibility], walk: &Walk<'_>) -> Globbed {
        self.0
            .iter()
            .filter(|glob| seen.contains(&glob.visibility))
            .map(|glob| glob.source.globbed(name, walk))
            .fold(Globbed::Absent, Globbed::joined)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Entries {
    declared: Declarations,
    imports: Imports,
    globs: Globs,
}

impl Entries {
    pub(crate) fn of_items(items: &[Item]) -> Entries {
        let uses: Vec<&ItemUse> = items.iter().filter_map(Entries::used).collect();
        let imported: Vec<(Imported, Visibility)> = uses
            .iter()
            .flat_map(|used| {
                UsePath::imported(used).into_iter().map(|imported| (imported, Visibility::from(&used.vis)))
            })
            .collect();
        Entries {
            declared: Declarations(items.iter().filter_map(Declared::of_item).collect()),
            imports: Imports(
                imported.iter().filter_map(|(imported, visibility)| imported.import(*visibility)).collect(),
            ),
            globs: Globs(
                imported
                    .iter()
                    .filter_map(|(imported, visibility)| imported.glob(*visibility))
                    .chain(items.iter().filter_map(Glob::of_item))
                    .collect(),
            ),
        }
    }

    pub(crate) fn of_block(statements: &[Stmt]) -> Entries {
        statements.iter().map(Entries::of_statement).fold(Entries::default(), Entries::merged)
    }

    pub(super) fn declare(&mut self, declared: Declared) {
        self.declared.0.push(declared);
    }

    pub(super) fn found(&self, level: ScopeId, name: &Name, walk: &Walk<'_>) -> Globbed {
        let seen = walk.seen(level);
        let at = walk.at_level(level);
        self.declared
            .found(level, name, seen)
            .or_else(|| self.imports.found(name, seen, &at))
            .or_else(|| self.globs.found(name, seen, &at))
    }

    fn of_statement(statement: &Stmt) -> Entries {
        match statement {
            Stmt::Item(item) => Entries::of_items(std::slice::from_ref(item)),
            Stmt::Macro(_) => Entries::expanded(),
            Stmt::Local(_) | Stmt::Expr(..) => Entries::default(),
        }
    }

    fn expanded() -> Entries {
        Entries { globs: Globs(vec![Glob::EXPANSION]), ..Entries::default() }
    }

    fn merged(mut self, other: Entries) -> Entries {
        self.declared.0.extend(other.declared.0);
        self.imports.0.extend(other.imports.0);
        self.globs.0.extend(other.globs.0);
        self
    }

    fn used(item: &Item) -> Option<&ItemUse> {
        match item {
            Item::Use(used) => Some(used),
            _ => None,
        }
    }
}
