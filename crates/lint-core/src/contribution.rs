use std::iter;
use std::rc::Rc;

use algebra::{Monoid, Semigroup};
use syntax::Node;

use crate::kind_index::{Cursors, Indexed, Kinds};
use crate::macros::Macros;
use crate::source_file::SourceFile;
use crate::watch::Descend;
use crate::watch::{Contribute, Kind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope(Kind);

#[derive(Debug)]
pub struct Contributed<Open, Part> {
    within: Contribution<Open, Part>,
    descend: Descend,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Contribution<Open, Part> {
    open: Open,
    part: Part,
}

#[derive(Debug)]
pub struct Contributing<Contributor: Contribute> {
    contributor: Contributor,
    kinds: Vec<Kind>,
    mask: Kinds,
}

#[derive(Debug, Default)]
pub(crate) struct Walked<Open, Part> {
    contribution: Contribution<Open, Part>,
    entered: WalkPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalkPosition(usize);

#[derive(Debug)]
struct Frame<'up, Open> {
    open: &'up Open,
    enclosing: Option<&'up Frame<'up, Open>>,
}

#[derive(Debug)]
pub struct Around<'up, Open> {
    frame: Option<&'up Frame<'up, Open>>,
    position: WalkPosition,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Visit<'node, 'file, 'index> {
    file: Option<&'file SourceFile>,
    kind: Kind,
    node: Node<'node>,
    indexed: Option<Indexed<'index>>,
}

impl<'node, 'file, 'index> Visit<'node, 'file, 'index> {
    pub(crate) fn new(
        file: Option<&'file SourceFile>,
        kind: Kind,
        node: Node<'node>,
        indexed: Option<Indexed<'index>>,
    ) -> Visit<'node, 'file, 'index> {
        Visit { file, kind, node, indexed }
    }

    fn expanded(self, descend: Descend) -> Rc<[syntax::Stmt]> {
        let invocation = descend.expanding(Visit::invocation(self.node));
        self.file.zip(invocation).map_or_else(|| Rc::from([]), |(source, called)| Macros::expanded(source, called))
    }

    fn invocation(node: Node<'node>) -> Option<&'node syntax::Macro> {
        match node {
            Node::Macro(invocation) => Some(invocation),
            _ => None,
        }
    }
}

impl<Open: Monoid, Part: Monoid> Contributed<Open, Part> {
    #[must_use]
    pub fn new(open: Open, part: Part, descend: Descend) -> Contributed<Open, Part> {
        Contributed { within: Contribution { open, part }, descend }
    }

    #[must_use]
    pub fn parted(part: Part) -> Contributed<Open, Part> {
        Contributed::new(Open::default(), part, Descend::Into)
    }

    #[must_use]
    pub fn descending(descend: Descend) -> Contributed<Open, Part> {
        Contributed::new(Open::default(), Part::default(), descend)
    }
}

impl<'up, Open> Around<'up, Open> {
    #[must_use]
    pub fn position(&self) -> WalkPosition {
        self.position
    }

    pub fn nearest(&self) -> impl Iterator<Item = &'up Open> {
        iter::successors(self.frame, |frame| frame.enclosing).map(|frame| frame.open)
    }
}

impl<Open, Part> Contribution<Open, Part> {
    pub fn concluded<Out>(self, into: impl FnOnce(Open, Part) -> Out) -> Out {
        into(self.open, self.part)
    }
}

impl<Open: Semigroup, Part: Semigroup> Semigroup for Contribution<Open, Part> {
    fn combine(self, other: Contribution<Open, Part>) -> Contribution<Open, Part> {
        Contribution { open: self.open.combine(other.open), part: self.part.combine(other.part) }
    }
}

impl<Open: Monoid, Part: Monoid> Monoid for Contribution<Open, Part> {}

impl<Open: Semigroup, Part: Semigroup> Semigroup for Walked<Open, Part> {
    fn combine(self, other: Walked<Open, Part>) -> Walked<Open, Part> {
        Walked {
            contribution: self.contribution.combine(other.contribution),
            entered: self.entered.combine(other.entered),
        }
    }
}

impl<Open: Monoid, Part: Monoid> Monoid for Walked<Open, Part> {}

impl<Contributor: Contribute> Contributing<Contributor> {
    pub fn new(contributor: Contributor) -> Contributing<Contributor> {
        let kinds: Vec<Kind> =
            contributor.kinds().iter().copied().chain(contributor.scopes().iter().map(|scope| scope.0)).collect();
        let mask = Kinds::listed(&kinds);
        Contributing { contributor, kinds, mask }
    }

    pub fn concluded<Out>(
        self,
        file: &SourceFile,
        into: impl FnOnce(Contributor, Contributor::Open, Contributor::Part) -> Out,
    ) -> Out {
        let Contribution { open, part } = self.contributed(file);
        into(self.contributor, open, part)
    }

    #[must_use]
    pub fn contributed(&self, file: &SourceFile) -> Contribution<Contributor::Open, Contributor::Part> {
        let top = Around { frame: None, position: WalkPosition::default() };
        let cursors = Some(file.kind_index().top());
        self.siblings(Some(file), Node::top_level(file.syntax()), cursors, &top).contribution
    }

    #[must_use]
    pub fn contributed_items<'item>(
        &self,
        items: impl IntoIterator<Item = &'item syntax::Item>,
    ) -> Contribution<Contributor::Open, Contributor::Part> {
        let top = Around { frame: None, position: WalkPosition::default() };
        self.siblings(None, items.into_iter().map(Node::Item), None, &top).contribution
    }

    pub fn contributor(&self) -> &Contributor {
        &self.contributor
    }

    pub(crate) fn siblings<'node>(
        &self,
        file: Option<&SourceFile>,
        nodes: impl IntoIterator<Item = Node<'node>>,
        cursors: Option<Cursors<'_>>,
        around: &Around<'_, Contributor::Open>,
    ) -> Walked<Contributor::Open, Contributor::Part> {
        let positions = cursors.into_iter().flatten().map(Some).chain(iter::repeat(None));
        nodes.into_iter().zip(positions).fold(Walked::default(), |walked, (node, cursor)| {
            let next = self.walked(
                file,
                node,
                cursor,
                &Around { frame: around.frame, position: around.position.combine(walked.entered) },
            );
            walked.combine(next)
        })
    }

    fn walked(
        &self,
        file: Option<&SourceFile>,
        node: Node<'_>,
        indexed: Option<Indexed<'_>>,
        around: &Around<'_, Contributor::Open>,
    ) -> Walked<Contributor::Open, Contributor::Part> {
        let relevant = indexed.map_or(Some(()), |cursor| cursor.watches(self.mask));
        relevant.map_or_default(|()| self.dispatched(file, node, indexed, around))
    }

    pub(crate) fn unwatched(
        &self,
        file: Option<&SourceFile>,
        node: Node<'_>,
        cursors: Option<Cursors<'_>>,
        around: &Around<'_, Contributor::Open>,
    ) -> Walked<Contributor::Open, Contributor::Part> {
        self.siblings(file, node.children(), cursors, around)
    }

    pub(crate) fn watches(&self, kind: Kind) -> Option<Kind> {
        self.kinds.contains(&kind).then_some(kind)
    }

    pub(crate) fn entered<On, Off>(
        &self,
        visit: Visit<'_, '_, '_>,
        around: &Around<'_, Contributor::Open>,
        on: On,
        off: Off,
    ) -> Walked<Contributor::Open, Contributor::Part>
    where
        On: FnOnce(&Contributor, &Around<'_, Contributor::Open>) -> Contributed<Contributor::Open, Contributor::Part>,
        Off: FnOnce(&Contributor, Contribution<Contributor::Open, Contributor::Part>) -> Contributor::Part,
    {
        let Contributed { within, descend } = on(&self.contributor, around);
        let scoping = Scoping::among(self.contributor.scopes(), visit.kind);
        let frame = Frame { open: &within.open, enclosing: around.frame };
        let inner = Around {
            frame: scoping.framed(&frame).or(around.frame),
            position: around.position.combine(WalkPosition::ONE),
        };
        let children = descend.descending(visit.node).map_or_default(|node| {
            self.siblings(visit.file, node.children(), visit.indexed.map(Indexed::children), &inner)
        });
        let after = Around { frame: inner.frame, position: inner.position.combine(children.entered) };
        let expansion = self.siblings(visit.file, visit.expanded(descend).iter().map(Node::Stmt), None, &after);
        let below = children.combine(expansion);
        let entered = below.entered.combine(WalkPosition::ONE);
        let merged = within.combine(below.contribution);
        scoping.walked(merged, entered, |inside| off(&self.contributor, inside))
    }
}

#[derive(Clone, Copy, Debug)]
enum Scoping {
    Scoped,
    Flat,
}

impl Scoping {
    fn among(scopes: &[Scope], kind: Kind) -> Scoping {
        scopes.iter().find(|scope| scope.0 == kind).map_or(Scoping::Flat, |_| Scoping::Scoped)
    }

    fn framed<Frame>(self, frame: Frame) -> Option<Frame> {
        match self {
            Scoping::Scoped => Some(frame),
            Scoping::Flat => None,
        }
    }

    fn walked<Open: Default, Part>(
        self,
        merged: Contribution<Open, Part>,
        entered: WalkPosition,
        off: impl FnOnce(Contribution<Open, Part>) -> Part,
    ) -> Walked<Open, Part> {
        match self {
            Scoping::Scoped => {
                Walked { contribution: Contribution { open: Open::default(), part: off(merged) }, entered }
            }
            Scoping::Flat => Walked { contribution: merged, entered },
        }
    }
}

impl WalkPosition {
    const ONE: WalkPosition = WalkPosition(1);
}

impl Scope {
    #[must_use]
    pub const fn new(kind: Kind) -> Scope {
        Scope(kind)
    }
}

impl Semigroup for WalkPosition {
    fn combine(self, other: WalkPosition) -> WalkPosition {
        WalkPosition(self.0 + other.0)
    }
}

impl Monoid for WalkPosition {}
