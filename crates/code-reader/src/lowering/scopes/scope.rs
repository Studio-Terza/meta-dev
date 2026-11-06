use syn::{Item, UseTree};

use code_model::Resolved;
use lint_layout::{Name, Word};

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
    traits: Vec<Name>,
}

#[derive(Debug)]
struct Binding {
    name: Name,
    path: Vec<Name>,
}

impl Scope {
    pub(super) fn new<'item>(reach: Reach, items: impl IntoIterator<Item = &'item Item>) -> Scope {
        let items: Vec<&Item> = items.into_iter().collect();
        Scope {
            reach,
            bindings: items.iter().flat_map(|item| Binding::imported(item)).collect(),
            traits: items.iter().filter_map(|item| Self::declared(item)).collect(),
        }
    }

    pub(super) fn reach(&self) -> Reach {
        self.reach
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

impl Binding {
    fn imported(item: &Item) -> Vec<Binding> {
        match item {
            Item::Use(used) => Self::under(&[], &used.tree),
            _ => Vec::new(),
        }
    }

    fn under(prefix: &[Name], tree: &UseTree) -> Vec<Binding> {
        match tree {
            UseTree::Path(path) => Self::under(&[prefix, &[Name::from(&path.ident)]].concat(), &path.tree),
            UseTree::Name(named) => vec![Binding::new(Name::from(&named.ident), prefix, Name::from(&named.ident))],
            UseTree::Rename(renamed) => {
                vec![Binding::new(Name::from(&renamed.rename), prefix, Name::from(&renamed.ident))]
            }
            UseTree::Group(group) => group.items.iter().flat_map(|tree| Self::under(prefix, tree)).collect(),
            UseTree::Glob(_) => Vec::new(),
        }
    }

    fn new(name: Name, prefix: &[Name], last: Name) -> Binding {
        Binding { name, path: prefix.iter().cloned().chain(std::iter::once(last)).collect() }
    }
}
