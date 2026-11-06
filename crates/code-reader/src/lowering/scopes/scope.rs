use syn::{Ident, Item, UseTree};

use code_model::Resolved;
use lint_layout::{Name, Whole, Word, words};

use super::root::Root;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::lowering) enum Reach {
    Module,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Meaning<'scope> {
    Written,
    Bound(&'scope [Name]),
    Declared,
    Prelude(&'static [Word<'static>]),
}

impl Meaning<'_> {
    pub(super) fn resolved(self, path: &[Name]) -> Resolved {
        match self {
            Meaning::Written => Root::rooted(path),
            Meaning::Bound(bound) => Root::rooted(&[bound, path.get(1..).unwrap_or_default()].concat()),
            Meaning::Declared => Resolved::Crate(Root::prefixed(Root::CRATE, path)),
            Meaning::Prelude(canonical) => Resolved::Std(canonical.iter().copied().map(Name::from).collect()),
        }
    }
}

#[derive(Debug)]
pub(super) struct Scope {
    reach: Reach,
    bindings: Vec<Binding>,
    globs: Vec<Vec<Name>>,
    traits: Vec<Name>,
}

#[derive(Clone, Debug)]
struct Binding {
    name: Name,
    path: Vec<Name>,
}

impl Scope {
    pub(super) fn new<'item>(reach: Reach, items: impl IntoIterator<Item = &'item Item>) -> Scope {
        let items: Vec<&Item> = items.into_iter().collect();
        let imports: Vec<Import> = items.iter().flat_map(|item| Import::from_item(item)).collect();
        Scope {
            reach,
            bindings: imports.iter().filter_map(Import::binding).collect(),
            globs: imports.iter().filter_map(Import::glob).collect(),
            traits: items.iter().filter_map(|item| Self::declared(item)).collect(),
        }
    }

    pub(super) fn reach(&self) -> Reach {
        self.reach
    }

    pub(super) fn globs(&self) -> &[Vec<Name>] {
        &self.globs
    }

    pub(super) fn meaning(&self, first: &Name) -> Option<Meaning<'_>> {
        self.bindings
            .iter()
            .find(|binding| binding.name == *first)
            .map(|binding| Meaning::Bound(&binding.path))
            .or_else(|| self.traits.iter().find(|declared| *declared == first).map(|_| Meaning::Declared))
    }

    fn declared(item: &Item) -> Option<Name> {
        match item {
            Item::Trait(declared) => Some(Name::from(&declared.ident)),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum Import {
    Named(Binding),
    Glob(Vec<Name>),
}

impl Import {
    fn from_item(item: &Item) -> Vec<Import> {
        match item {
            Item::Use(used) => Self::under(&[], &used.tree),
            _ => Vec::new(),
        }
    }

    fn binding(&self) -> Option<Binding> {
        match self {
            Import::Named(binding) => Some(binding.clone()),
            Import::Glob(_) => None,
        }
    }

    fn glob(&self) -> Option<Vec<Name>> {
        match self {
            Import::Glob(path) => Some(path.clone()),
            Import::Named(_) => None,
        }
    }

    fn under(prefix: &[Name], tree: &UseTree) -> Vec<Import> {
        match tree {
            UseTree::Path(path) => Self::under(&[prefix, &[Name::from(&path.ident)]].concat(), &path.tree),
            UseTree::Name(named) => Leaf::from(&named.ident).imports(prefix, &named.ident),
            UseTree::Rename(renamed) => {
                vec![Import::Named(Binding::new(Name::from(&renamed.rename), prefix, Name::from(&renamed.ident)))]
            }
            UseTree::Group(group) => group.items.iter().flat_map(|tree| Self::under(prefix, tree)).collect(),
            UseTree::Glob(_) => vec![Import::Glob(prefix.to_vec())],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum Leaf {
    Itself,
    #[default]
    Named,
}

impl Leaf {
    const SELF: [Word<'static>; 1] = words!["self"];

    fn imports(self, prefix: &[Name], ident: &Ident) -> Vec<Import> {
        match self {
            Leaf::Itself => prefix
                .split_last()
                .map(|(last, parent)| Import::Named(Binding::new(last.clone(), parent, last.clone())))
                .into_iter()
                .collect(),
            Leaf::Named => vec![Import::Named(Binding::new(Name::from(ident), prefix, Name::from(ident)))],
        }
    }
}

impl From<&Ident> for Leaf {
    fn from(ident: &Ident) -> Leaf {
        Name::from(ident).matches::<Whole>(&Leaf::SELF).map(|_| Leaf::Itself).unwrap_or_default()
    }
}

impl Binding {
    fn new(name: Name, prefix: &[Name], last: Name) -> Binding {
        Binding { name, path: prefix.iter().cloned().chain(std::iter::once(last)).collect() }
    }
}
