use std::fmt;
use std::iter;
use std::rc::Rc;

use syntax::Node;

use crate::joins::Joins;
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
    mask: Kinds,
}

#[derive(Debug)]
pub struct Tallied<Contributor: Contribute> {
    contributor: Contributor,
    contribution: Contribution<Contributor::Open, Contributor::Part>,
    visits: Visits,
}

impl<Contributor: Contribute> Tallied<Contributor> {
    pub fn concluded<Out>(
        self,
        into: impl FnOnce(Contributor, Contributor::Open, Contributor::Part, Visits) -> Out,
    ) -> Out {
        let Contribution { open, part } = self.contribution;
        into(self.contributor, open, part, self.visits)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Walked<Open, Part> {
    contribution: Contribution<Open, Part>,
    entered: WalkPosition,
    visits: Visits,
}

#[derive(Debug, Default)]
struct Stepping<'index, Open, Part> {
    walked: Walked<Open, Part>,
    cursors: Option<Cursors<'index>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalkPosition(usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Visits(usize);

impl Visits {
    const ONE: Visits = Visits(1);
}

impl Joins for Visits {
    fn combine(self, other: Visits) -> Visits {
        Visits(self.0 + other.0)
    }
}

impl fmt::Display for Visits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

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

impl<Open: Joins, Part: Joins> Contributed<Open, Part> {
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

impl<Open: Joins, Part: Joins> Joins for Contribution<Open, Part> {
    fn combine(self, other: Contribution<Open, Part>) -> Contribution<Open, Part> {
        Contribution { open: self.open.combine(other.open), part: self.part.combine(other.part) }
    }
}

impl<Open: Joins, Part: Joins> Joins for Walked<Open, Part> {
    fn combine(self, other: Walked<Open, Part>) -> Walked<Open, Part> {
        Walked {
            contribution: self.contribution.combine(other.contribution),
            entered: self.entered.combine(other.entered),
            visits: self.visits.combine(other.visits),
        }
    }
}

impl<Open: Joins, Part: Joins> From<Visits> for Walked<Open, Part> {
    fn from(visits: Visits) -> Walked<Open, Part> {
        Walked { visits, ..Walked::default() }
    }
}

impl<Contributor: Contribute> Contributing<Contributor> {
    pub fn new(contributor: Contributor) -> Contributing<Contributor> {
        let mask =
            Kinds::listed(contributor.kinds().iter().copied().chain(contributor.scopes().iter().map(|scope| scope.0)));
        Contributing { contributor, mask }
    }

    pub fn concluded<Out>(
        self,
        file: &SourceFile,
        into: impl FnOnce(Contributor, Contributor::Open, Contributor::Part) -> Out,
    ) -> Out {
        let Contribution { open, part } = self.contributed(file);
        into(self.contributor, open, part)
    }

    pub fn tallied(self, file: &SourceFile) -> Tallied<Contributor> {
        let Walked { contribution, visits, .. } = self.walked_file(file);
        Tallied { contributor: self.contributor, contribution, visits }
    }

    #[must_use]
    pub fn contributed(&self, file: &SourceFile) -> Contribution<Contributor::Open, Contributor::Part> {
        self.walked_file(file).contribution
    }

    fn walked_file(&self, file: &SourceFile) -> Walked<Contributor::Open, Contributor::Part> {
        let top = Around { frame: None, position: WalkPosition::default() };
        let cursors = Some(file.kind_index().top());
        self.siblings(Some(file), Node::top_level(file.syntax()), cursors, &top)
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
        let unindexed = cursors.as_ref().map_or(Some(()), |_| None);
        let fallback = iter::repeat(None).map_while(|none| unindexed.map(|()| none));
        let positions = cursors.into_iter().flatten().map(Some).chain(fallback);
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
        relevant.map_or_default(|()| Walked::from(Visits::ONE).combine(self.dispatched(file, node, indexed, around)))
    }

    pub(crate) fn unwatched(
        &self,
        file: Option<&SourceFile>,
        node: Node<'_>,
        cursors: Option<Cursors<'_>>,
        around: &Around<'_, Contributor::Open>,
    ) -> Walked<Contributor::Open, Contributor::Part> {
        let start = Stepping { walked: Walked::default(), cursors };
        node.folded(start, |Stepping { walked, cursors: mut rest }, child| {
            let cursor = rest.as_mut().and_then(Iterator::next);
            let position = around.position.combine(walked.entered);
            let next = self.walked(file, child, cursor, &Around { frame: around.frame, position });
            Stepping { walked: walked.combine(next), cursors: rest }
        })
        .walked
    }

    pub(crate) fn watches(&self, kind: Kind) -> Option<Kind> {
        self.mask.overlap(Kinds::from(kind)).map(|()| kind)
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
        let children = descend
            .descending(visit.node)
            .map_or_default(|node| self.unwatched(visit.file, node, visit.indexed.map(Indexed::children), &inner));
        let after = Around { frame: inner.frame, position: inner.position.combine(children.entered) };
        let expansion = self.siblings(visit.file, visit.expanded(descend).iter().map(Node::Stmt), None, &after);
        let below = children.combine(expansion);
        let entered = below.entered.combine(WalkPosition::ONE);
        let merged = within.combine(below.contribution);
        scoping.walked(merged, entered, |inside| off(&self.contributor, inside)).combine(Walked::from(below.visits))
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
            Scoping::Scoped => Walked {
                contribution: Contribution { open: Open::default(), part: off(merged) },
                entered,
                visits: Visits::default(),
            },
            Scoping::Flat => Walked { contribution: merged, entered, visits: Visits::default() },
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

impl Joins for WalkPosition {
    fn combine(self, other: WalkPosition) -> WalkPosition {
        WalkPosition(self.0 + other.0)
    }
}
