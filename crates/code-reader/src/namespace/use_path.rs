use syn::{ItemUse, Path, Token, UseTree};

use code_model::ItemPath;
use lint_layout::{Name, Whole, Word, words};

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use super::entries::Imported;
use super::resolver::Walk;
use super::std_names::{Canonical, StdNames};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Leading {
    Relative,
    Global,
}

impl From<&Option<Token![::]>> for Leading {
    fn from(colons: &Option<Token![::]>) -> Leading {
        match colons {
            Some(_) => Leading::Global,
            None => Leading::Relative,
        }
    }
}

impl Leading {
    fn head(self, first: &Name, rest: &[Name]) -> Head {
        match self {
            Leading::Global => Head::Global(first.clone()),
            Leading::Relative => Head::relative(first, rest),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    Single,
    Multiple,
}

impl From<&[Name]> for Shape {
    fn from(rest: &[Name]) -> Shape {
        match rest.first() {
            Some(_) => Shape::Multiple,
            None => Shape::Single,
        }
    }
}

impl Shape {
    pub(super) fn fallback(self, first: &Name, space: &Namespace) -> Binding {
        space
            .workspace(first)
            .map(Binding::Module)
            .or_else(|| StdNames::root(first))
            .or_else(|| self.prelude(first).map(Binding::Std))
            .unwrap_or_else(|| self.unbound(first))
    }

    fn prelude(self, first: &Name) -> Option<ItemPath> {
        match self {
            Shape::Single => StdNames::prelude(first).map(Canonical::path),
            Shape::Multiple => None,
        }
    }

    fn unbound(self, first: &Name) -> Binding {
        match self {
            Shape::Single => Binding::Unknown,
            Shape::Multiple => Binding::External(ItemPath::from_iter([first.clone()])),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Head {
    Empty,
    Global(Name),
    Anchored(Anchor),
    Named(Name, Shape),
}

impl Head {
    pub(super) fn started(self, walk: &Walk<'_>) -> Binding {
        match self {
            Head::Empty => Binding::Unknown,
            Head::Global(first) => walk.space().global(&first),
            Head::Anchored(anchor) => Binding::from(anchor.anchored(walk.space().anchors(walk.scope()))),
            Head::Named(first, shape) => walk.named(&first, shape),
        }
    }

    fn relative(first: &Name, rest: &[Name]) -> Head {
        match Anchor::spelled(first) {
            Some(anchor) => Head::Anchored(anchor),
            None => Head::Named(first.clone(), Shape::from(rest)),
        }
    }
}

#[derive(Debug)]
enum Leaf {
    Itself,
    Discarded,
    Named(Name),
}

impl From<Name> for Leaf {
    fn from(name: Name) -> Leaf {
        name.matches::<Whole>(&UsePath::SELF)
            .map(|_| Leaf::Itself)
            .or_else(|| name.matches::<Whole>(&UsePath::DISCARDED).map(|_| Leaf::Discarded))
            .unwrap_or(Leaf::Named(name))
    }
}

impl Leaf {
    fn path(self, prefix: &UsePath) -> Option<UsePath> {
        match self {
            Leaf::Itself => prefix.segments.last().map(|_| prefix.clone()),
            Leaf::Named(name) => Some(prefix.joined(name)),
            Leaf::Discarded => None,
        }
    }

    fn bound(self, prefix: &UsePath) -> Option<Name> {
        match self {
            Leaf::Itself => prefix.segments.last().cloned(),
            Leaf::Named(name) => Some(name),
            Leaf::Discarded => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UsePath {
    leading: Leading,
    segments: Vec<Name>,
}

impl UsePath {
    const SELF: [Word<'static>; 1] = words!["self"];
    const DISCARDED: [Word<'static>; 1] = words!["_"];

    pub(super) fn imported(used: &ItemUse) -> Vec<Imported> {
        UsePath { leading: Leading::from(&used.leading_colon), segments: Vec::new() }.under(&used.tree)
    }

    pub(super) fn head(&self) -> Head {
        match self.segments.split_first() {
            Some((first, rest)) => self.leading.head(first, rest),
            None => Head::Empty,
        }
    }

    pub(super) fn tail(&self) -> &[Name] {
        self.segments.get(1..).unwrap_or_default()
    }

    pub(super) fn written(&self) -> ItemPath {
        self.segments.iter().cloned().collect()
    }

    fn joined(&self, segment: Name) -> UsePath {
        UsePath { leading: self.leading, segments: self.segments.iter().cloned().chain([segment]).collect() }
    }

    fn under(&self, tree: &UseTree) -> Vec<Imported> {
        match tree {
            UseTree::Path(path) => self.joined(Name::from(&path.ident)).under(&path.tree),
            UseTree::Name(named) => self.named(Name::from(&named.ident), Name::from(&named.ident)),
            UseTree::Rename(renamed) => self.named(Name::from(&renamed.ident), Name::from(&renamed.rename)),
            UseTree::Group(group) => group.items.iter().flat_map(|tree| self.under(tree)).collect(),
            UseTree::Glob(_) => vec![Imported::Glob(self.clone())],
        }
    }

    fn named(&self, ident: Name, binds: Name) -> Vec<Imported> {
        Leaf::from(ident)
            .path(self)
            .zip(Leaf::from(binds).bound(self))
            .map(|(path, bound)| Imported::Named(bound, path))
            .into_iter()
            .collect()
    }
}

impl From<&Path> for UsePath {
    fn from(path: &Path) -> UsePath {
        UsePath {
            leading: Leading::from(&path.leading_colon),
            segments: path.segments.iter().map(|segment| Name::from(&segment.ident)).collect(),
        }
    }
}
