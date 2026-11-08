use syn::{Ident, ItemUse, Path, Token, UseTree};

use std::collections::BTreeSet;

use code_model::{Domain, ItemPath, Primitive, Reexport};
use lint_layout::{Dependencies, Name, Whole, Word, words};

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::ScopeId;
use super::entries::Imported;
use super::resolver::{Outer, Resolution, Resolver};
use super::std_names::{Canonical, StdNames, Table};
use super::view::View;

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
    fn head<'path>(self, first: &'path Name, rest: &[Name], reading: Reading) -> Head<'path> {
        match self {
            Leading::Global => Head::Global(first),
            Leading::Relative => Head::relative(first, rest, reading),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reading {
    Written,
    Imported,
    Explicit,
}

impl Reading {
    pub(super) fn seen(self, view: View<'_>) -> View<'_> {
        match self {
            Reading::Written => view,
            Reading::Imported => view.undisputed(),
            Reading::Explicit => View::ABSENT,
        }
    }

    fn lone(self, external: Binding) -> Binding {
        match self {
            Reading::Written | Reading::Explicit => Binding::UNBOUND,
            Reading::Imported => external,
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
    pub(super) fn fallback(self, first: &Name, reading: Reading, space: &Namespace, viewer: ScopeId) -> Outer {
        let prelude = space.prelude(first, viewer);
        let known = self
            .primitive(first)
            .map(Binding::Primitive)
            .or(prelude)
            .or_else(|| space.workspace(first).map(Binding::Module))
            .or_else(|| StdNames::root(first))
            .or_else(|| self.prelude(first).map(Binding::Std));
        Outer::known_or(known, || self.unbound(reading, Binding::external(first, space.dependencies(viewer))))
    }

    pub(super) fn primitive(self, first: &Name) -> Option<Primitive> {
        match self {
            Shape::Single => StdNames::primitive(first),
            Shape::Multiple => None,
        }
    }

    fn prelude(self, first: &Name) -> Option<ItemPath> {
        match self {
            Shape::Single => StdNames::canonical(Table::Prelude, first).map(Canonical::path),
            Shape::Multiple => None,
        }
    }

    fn looked(self, domain: Domain) -> Domain {
        match self {
            Shape::Single => domain,
            Shape::Multiple => Domain::Types,
        }
    }

    fn unbound(self, reading: Reading, external: Binding) -> Binding {
        match self {
            Shape::Single => reading.lone(external),
            Shape::Multiple => external,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Relative<'path> {
    first: &'path Name,
    shape: Shape,
    reading: Reading,
}

impl Relative<'_> {
    fn reexported(self, written: &ItemPath, dependencies: &Dependencies, declared: &BTreeSet<&Name>) -> Reexport {
        declared.get(self.first).map_or_else(
            || Reexport::dependency(dependencies.package(self.first), written),
            |_| Reexport::own(written.as_ref(), declared),
        )
    }

    fn started(self, resolver: &Resolver<'_, '_>, viewer: ScopeId, domain: Domain) -> Resolution {
        match self.shape.looked(domain) {
            Domain::Types => resolver.named(self.first, self.shape, self.reading, viewer),
            Domain::Macros => resolver.macro_named(self.first, viewer),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Head<'path> {
    Empty,
    Global(&'path Name),
    Anchored(Anchor),
    Named(Relative<'path>),
}

impl Head<'_> {
    pub(super) fn started(self, resolver: &Resolver<'_, '_>, viewer: ScopeId, domain: Domain) -> Resolution {
        match self {
            Head::Empty => Resolution::Bound(Binding::UNBOUND),
            Head::Global(first) => {
                Resolution::Bound(resolver.space().global(first, resolver.space().dependencies(viewer)))
            }
            Head::Anchored(anchor) => resolver.anchored(anchor, viewer),
            Head::Named(relative) => relative.started(resolver, viewer, domain),
        }
    }

    fn reexported(self, written: &ItemPath, dependencies: &Dependencies, declared: &BTreeSet<&Name>) -> Reexport {
        match self {
            Head::Empty => Reexport::Opaque,
            Head::Global(first) => Reexport::dependency(dependencies.package(first), written),
            Head::Anchored(anchor) => anchor.reexported(written, declared),
            Head::Named(relative) => relative.reexported(written, dependencies, declared),
        }
    }

    fn relative<'path>(first: &'path Name, rest: &[Name], reading: Reading) -> Head<'path> {
        match Anchor::spelled(first) {
            Some(anchor) => Head::Anchored(anchor),
            None => Head::Named(Relative { first, shape: Shape::from(rest), reading }),
        }
    }
}

#[derive(Debug)]
pub(super) enum Leaf {
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
    pub(super) fn standalone(self) -> Option<Name> {
        match self {
            Leaf::Named(name) => Some(name),
            Leaf::Itself | Leaf::Discarded => None,
        }
    }

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
    reading: Reading,
    segments: Vec<Name>,
}

impl UsePath {
    const SELF: [Word<'static>; 1] = words!["self"];
    const DISCARDED: [Word<'static>; 1] = words!["_"];

    pub(super) fn imported(used: &ItemUse) -> Vec<Imported> {
        UsePath { leading: Leading::from(&used.leading_colon), reading: Reading::Imported, segments: Vec::new() }
            .under(&used.tree)
    }

    pub(super) fn head(&self) -> Head<'_> {
        match self.segments.split_first() {
            Some((first, rest)) => self.leading.head(first, rest, self.reading),
            None => Head::Empty,
        }
    }

    pub(super) fn tail(&self) -> &[Name] {
        self.segments.get(1..).unwrap_or_default()
    }

    pub(super) fn exported(krate: &Name, entry: &Name) -> UsePath {
        UsePath { leading: Leading::Global, reading: Reading::Written, segments: vec![krate.clone(), entry.clone()] }
    }

    pub(crate) fn written(&self) -> ItemPath {
        self.segments.iter().cloned().collect()
    }

    pub(super) fn global(krate: &Name) -> UsePath {
        UsePath { leading: Leading::Global, reading: Reading::Written, segments: vec![krate.clone()] }
    }

    pub(super) fn reexport(&self, dependencies: &Dependencies, declared: &BTreeSet<&Name>) -> Reexport {
        self.head().reexported(&self.written(), dependencies, declared)
    }

    fn joined(&self, segment: Name) -> UsePath {
        UsePath {
            leading: self.leading,
            reading: self.reading,
            segments: self.segments.iter().cloned().chain([segment]).collect(),
        }
    }

    fn under(&self, tree: &UseTree) -> Vec<Imported> {
        match tree {
            UseTree::Path(path) => self.joined(Name::from(&path.ident)).under(&path.tree),
            UseTree::Name(named) => self.named(Name::from(&named.ident), Name::from(&named.ident)),
            UseTree::Rename(renamed) => self.named(Name::from(&renamed.ident), Name::from(&renamed.rename)),
            UseTree::Group(group) => group.items.iter().flat_map(|member| self.under(member)).collect(),
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

impl From<&Ident> for UsePath {
    fn from(ident: &Ident) -> UsePath {
        UsePath { leading: Leading::Relative, reading: Reading::Written, segments: vec![Name::from(ident)] }
    }
}

impl From<&Path> for UsePath {
    fn from(path: &Path) -> UsePath {
        UsePath {
            leading: Leading::from(&path.leading_colon),
            reading: Reading::Written,
            segments: path.segments.iter().map(|segment| Name::from(&segment.ident)).collect(),
        }
    }
}
