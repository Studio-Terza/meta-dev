use std::iter;
use std::mem;

use algebra::{Monoid, Semigroup};
use lint_layout::Name;
use syntax::{Ident, Macro, Node};

use super::{Around, Contributed, Contributing, Contribution, Scoping, Tallied, Visit, Visits, WalkPosition, Walked};
use crate::kind_index::{Cursors, Indexed, Kinds};
use crate::macros::Macros;
use crate::source_file::SourceFile;
use crate::watch::{Contribute, Descend, Facts, Kind};

pub trait Hook {
    type Facts<'node>
    where
        Self: 'node;

    fn node(&self) -> Node<'_>;

    fn facts(&self) -> Self::Facts<'_>;

    fn opened<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        facts: &Self::Facts<'_>,
        around: &Around<'_, Contributor::Open>,
    ) -> Contributed<Contributor::Open, Contributor::Part>;

    fn closed<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        facts: &Self::Facts<'_>,
        inner: Contribution<Contributor::Open, Contributor::Part>,
    ) -> Contributor::Part;
}

#[derive(Debug, Default)]
pub struct Joined<Head, Tail> {
    head: Head,
    tail: Tail,
}

#[derive(Clone, Copy, Debug)]
pub struct Unjoined;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Alive(u128);

#[derive(Clone, Copy, Debug, Default)]
pub struct Entered {
    alive: Alive,
    watched: Alive,
    descending: Alive,
    expanding: Alive,
}

#[derive(Debug)]
struct Entry<Open, Part> {
    part: Part,
    open: Option<Open>,
    scoping: Scoping,
    inner: WalkPosition,
    below: Walked<Open, Part>,
}

#[derive(Debug)]
struct Entries<Open, Part> {
    open: Vec<Entry<Open, Part>>,
    root: Walked<Open, Part>,
}

#[derive(Debug)]
struct Opens<Open>(Vec<Open>);

#[derive(Debug)]
pub struct Lane<Contributor: Contribute> {
    contributing: Contributing<Contributor>,
    entries: Entries<Contributor::Open, Contributor::Part>,
    opens: Opens<Contributor::Open>,
}

#[derive(Debug)]
pub(crate) struct Sweeping<'lanes, 'file, Lane> {
    lanes: &'lanes mut Lane,
    file: Option<&'file SourceFile>,
    mask: Kinds,
}

pub trait Joint: Sized {
    type Tallied;
    type Lanes: Lanes<Tallied = Self::Tallied>;

    fn lanes(self) -> Self::Lanes;

    fn swept(self, file: &SourceFile) -> Self::Tallied {
        let mut lanes = self.lanes();
        let mask = lanes.mask(Kinds::default());
        let alive = lanes.present(Alive::default());
        let mut sweeping = Sweeping { lanes: &mut lanes, file: Some(file), mask };
        sweeping.siblings(Node::top_level(file.syntax()), Some(file.kind_index().top()), alive);
        lanes.tallied()
    }
}

pub trait Lanes {
    const NEXT: Alive;

    type Tallied;

    fn mask(&self, kinds: Kinds) -> Kinds;

    fn present(&self, alive: Alive) -> Alive;

    fn relevant(&self, indexed: Indexed<'_>, alive: Alive, kept: Alive) -> Alive;

    fn visited(&mut self, alive: Alive);

    fn entered<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>, entered: Entered) -> Entered;

    fn exited<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>, watched: Alive);

    fn tallied(self) -> Self::Tallied;
}

impl<Head, Tail> Joined<Head, Tail> {
    pub fn new(head: Head, tail: Tail) -> Joined<Head, Tail> {
        Joined { head, tail }
    }

    pub fn concluded<Out>(self, into: impl FnOnce(Head, Tail) -> Out) -> Out {
        into(self.head, self.tail)
    }
}

impl Semigroup for Alive {
    fn combine(self, other: Alive) -> Alive {
        Alive(self.0 | other.0)
    }
}

impl Monoid for Alive {}

impl Alive {
    const FIRST: Alive = Alive(1);

    fn outside(self, watched: Alive) -> Alive {
        Alive(self.0 ^ (self.0 & watched.0))
    }

    fn holds(self, member: Alive) -> Option<()> {
        (self.0 & member.0 == member.0).then_some(())
    }

    fn live(self) -> Option<Alive> {
        (self.0 != 0).then_some(self)
    }

    fn entering(self, descend: Descend) -> Entered {
        Entered {
            alive: Alive::default(),
            watched: self,
            descending: descend.descending(self).unwrap_or_default(),
            expanding: descend.expanding(Some(self)).unwrap_or_default(),
        }
    }
}

impl From<Alive> for Entered {
    fn from(alive: Alive) -> Entered {
        Entered { alive, ..Entered::default() }
    }
}

impl Semigroup for Entered {
    fn combine(self, other: Entered) -> Entered {
        Entered {
            alive: self.alive.combine(other.alive),
            watched: self.watched.combine(other.watched),
            descending: self.descending.combine(other.descending),
            expanding: self.expanding.combine(other.expanding),
        }
    }
}

impl Monoid for Entered {}

impl<Open: Monoid, Part: Monoid> Walked<Open, Part> {
    fn absorbed(&mut self, other: Walked<Open, Part>) {
        *self = mem::take(self).combine(other);
    }
}

impl<Open: Monoid, Part: Monoid> Entries<Open, Part> {
    fn sink(&mut self) -> &mut Walked<Open, Part> {
        self.open.last_mut().map_or(&mut self.root, |entry| &mut entry.below)
    }

    fn position(&self) -> WalkPosition {
        self.open.last().map_or(self.root.entered, |entry| entry.inner.combine(entry.below.entered))
    }
}

impl<Open> Opens<Open> {
    fn kept(&mut self, scoping: Scoping, open: Open) -> Option<Open> {
        match scoping {
            Scoping::Scoped => self.stacked(open),
            Scoping::Flat => Some(open),
        }
    }

    fn stacked(&mut self, open: Open) -> Option<Open> {
        self.0.push(open);
        None
    }
}

impl<Open: Default> Opens<Open> {
    fn reopened(&mut self, kept: Option<Open>) -> Open {
        kept.or_else(|| self.0.pop()).unwrap_or_default()
    }
}

impl<Contributor: Contribute> From<Contributing<Contributor>> for Lane<Contributor> {
    fn from(contributing: Contributing<Contributor>) -> Lane<Contributor> {
        let entries = Entries { open: Vec::new(), root: Walked::default() };
        Lane { contributing, entries, opens: Opens(Vec::new()) }
    }
}

impl<Contributor: Contribute> From<Lane<Contributor>> for Tallied<Contributor> {
    fn from(lane: Lane<Contributor>) -> Tallied<Contributor> {
        let Walked { contribution, visits, .. } = lane.entries.root;
        Tallied { contributor: lane.contributing.contributor, contribution, visits }
    }
}

impl<Contributor: Contribute> Lane<Contributor> {
    fn visited(&mut self) {
        self.entries.sink().absorbed(Walked::from(Visits::ONE));
    }

    fn entered<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>) -> Option<Descend> {
        let kind = self.contributing.watches(Kind::of(node.node()))?;
        let position = self.entries.position();
        let around = Around { frame: None, stacked: &self.opens.0, position };
        let Contributed { within, descend } = node.opened(&self.contributing.contributor, facts, &around);
        let scoping = Scoping::among(self.contributing.contributor.scopes(), kind);
        let Contribution { open, part } = within;
        let kept = self.opens.kept(scoping, open);
        let inner = position.combine(WalkPosition::ONE);
        self.entries.open.push(Entry { part, open: kept, scoping, inner, below: Walked::default() });
        Some(descend)
    }

    fn exited<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>) {
        let closed = self.entries.open.pop().map(|done| {
            let open = self.opens.reopened(done.open);
            let Walked { contribution, entered, visits } = done.below;
            let merged = Contribution { open, part: done.part }.combine(contribution);
            let contributor = &self.contributing.contributor;
            let walked = done
                .scoping
                .walked(merged, entered.combine(WalkPosition::ONE), |inside| node.closed(contributor, facts, inside));
            walked.combine(Walked::from(visits))
        });
        closed.map_or_default(|walked| self.entries.sink().absorbed(walked));
    }
}

impl<Contributor: Contribute, Tail: Joint> Joint for Joined<Option<Contributing<Contributor>>, Tail> {
    type Tallied = Joined<Option<Tallied<Contributor>>, Tail::Tallied>;
    type Lanes = Joined<Option<Lane<Contributor>>, Tail::Lanes>;

    fn lanes(self) -> Self::Lanes {
        Joined { head: self.head.map(Lane::from), tail: self.tail.lanes() }
    }
}

impl Joint for Unjoined {
    type Tallied = Unjoined;
    type Lanes = Unjoined;

    fn lanes(self) -> Unjoined {
        self
    }
}

impl<Contributor: Contribute, Tail: Lanes> Joined<Option<Lane<Contributor>>, Tail> {
    fn lane(&mut self, alive: Alive) -> Option<&mut Lane<Contributor>> {
        alive.holds(Tail::NEXT).and(self.head.as_mut())
    }
}

impl<Contributor: Contribute, Tail: Lanes> Lanes for Joined<Option<Lane<Contributor>>, Tail> {
    const NEXT: Alive = Alive(Tail::NEXT.0 << 1_u32);

    type Tallied = Joined<Option<Tallied<Contributor>>, Tail::Tallied>;

    fn mask(&self, kinds: Kinds) -> Kinds {
        self.tail.mask(kinds.combine(self.head.as_ref().map_or_default(|lane| lane.contributing.mask)))
    }

    fn present(&self, alive: Alive) -> Alive {
        self.tail.present(alive.combine(self.head.as_ref().map_or_default(|_| Tail::NEXT)))
    }

    fn relevant(&self, indexed: Indexed<'_>, alive: Alive, kept: Alive) -> Alive {
        let lane = alive.holds(Tail::NEXT).and(self.head.as_ref());
        let mine = lane.and_then(|found| indexed.watches(found.contributing.mask)).map(|()| Tail::NEXT);
        self.tail.relevant(indexed, alive, kept.combine(mine.unwrap_or_default()))
    }

    fn visited(&mut self, alive: Alive) {
        self.lane(alive).map_or_default(Lane::visited);
        self.tail.visited(alive);
    }

    fn entered<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>, entered: Entered) -> Entered {
        let descend = self.lane(entered.alive).and_then(|lane| lane.entered(node, facts));
        let mine = descend.map_or_default(|found| Tail::NEXT.entering(found));
        self.tail.entered(node, facts, entered.combine(mine))
    }

    fn exited<Hooked: Hook>(&mut self, node: &Hooked, facts: &Hooked::Facts<'_>, watched: Alive) {
        self.lane(watched).map_or_default(|lane| lane.exited(node, facts));
        self.tail.exited(node, facts, watched);
    }

    fn tallied(self) -> Self::Tallied {
        Joined { head: self.head.map(Tallied::from), tail: self.tail.tallied() }
    }
}

impl Lanes for Unjoined {
    const NEXT: Alive = Alive::FIRST;

    type Tallied = Unjoined;

    fn mask(&self, kinds: Kinds) -> Kinds {
        kinds
    }

    fn present(&self, alive: Alive) -> Alive {
        alive
    }

    fn relevant(&self, _: Indexed<'_>, _: Alive, kept: Alive) -> Alive {
        kept
    }

    fn visited(&mut self, _: Alive) {}

    fn entered<Hooked: Hook>(&mut self, _: &Hooked, _: &Hooked::Facts<'_>, entered: Entered) -> Entered {
        entered
    }

    fn exited<Hooked: Hook>(&mut self, _: &Hooked, _: &Hooked::Facts<'_>, _: Alive) {}

    fn tallied(self) -> Unjoined {
        self
    }
}

impl<Lane: Lanes> Sweeping<'_, '_, Lane> {
    fn siblings<'node>(
        &mut self,
        nodes: impl IntoIterator<Item = Node<'node>>,
        cursors: Option<Cursors<'_>>,
        alive: Alive,
    ) {
        let unindexed = cursors.as_ref().map_or(Some(()), |_| None);
        let fallback = iter::repeat(None).map_while(|none| unindexed.map(|()| none));
        let positions = cursors.into_iter().flatten().map(Some).chain(fallback);
        nodes.into_iter().zip(positions).for_each(|(node, cursor)| self.walked(node, cursor, alive));
    }

    fn children(&mut self, node: Node<'_>, cursors: Option<Cursors<'_>>, alive: Alive) {
        node.folded(cursors, |mut rest, child| {
            let cursor = rest.as_mut().and_then(Iterator::next);
            self.walked(child, cursor, alive);
            rest
        });
    }

    fn walked(&mut self, node: Node<'_>, indexed: Option<Indexed<'_>>, alive: Alive) {
        let mask = self.mask;
        let relevant = indexed
            .map_or(Some(alive), |cursor| {
                cursor.watches(mask).map(|()| self.lanes.relevant(cursor, alive, Alive::default()))
            })
            .and_then(Alive::live);
        relevant.map_or_default(|live| {
            self.lanes.visited(live);
            self.dispatched(node, indexed, live);
        });
    }

    pub(crate) fn macro_entered(&mut self, found: &Macro, indexed: Option<Indexed<'_>>, alive: Alive) {
        self.file.map_or_default(|source| Macros::pin(source, found));
        self.entered(found, indexed, alive);
    }

    pub(crate) fn ident_entered(&mut self, found: &Ident, indexed: Option<Indexed<'_>>, alive: Alive) {
        let watched = self.mask.overlap(Kinds::from(Kind::Ident));
        watched.map_or_default(|()| self.entered(found, indexed, alive));
    }

    pub(crate) fn entered(&mut self, hooked: &impl Hook, indexed: Option<Indexed<'_>>, alive: Alive) {
        let node = hooked.node();
        let facts = hooked.facts();
        let kind = Kind::of(node);
        let watching = self.mask.overlap(Kinds::from(kind));
        let entered = watching.map_or_default(|()| self.lanes.entered(hooked, &facts, Entered::from(alive)));
        let descending = alive.outside(entered.watched).combine(entered.descending);
        descending.live().map_or_default(|live| self.children(node, indexed.map(Indexed::children), live));
        entered.expanding.live().map_or_default(|live| {
            let statements = Visit::new(self.file, kind, node, None).expanded(Descend::Expand);
            self.siblings(statements.iter().map(Node::Stmt), None, live);
        });
        self.lanes.exited(hooked, &facts, entered.watched);
    }
}

impl Hook for Macro {
    type Facts<'node> = Facts<'node, Macro>;

    fn node(&self) -> Node<'_> {
        Node::Macro(self)
    }

    fn facts(&self) -> Facts<'_, Macro> {
        Facts::new(self)
    }

    fn opened<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        facts: &Facts<'_, Macro>,
        around: &Around<'_, Contributor::Open>,
    ) -> Contributed<Contributor::Open, Contributor::Part> {
        contributor.on_macro(self, facts, around)
    }

    fn closed<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        facts: &Facts<'_, Macro>,
        inner: Contribution<Contributor::Open, Contributor::Part>,
    ) -> Contributor::Part {
        contributor.off_macro(self, facts, inner)
    }
}

impl Hook for Ident {
    type Facts<'node> = Name;

    fn node(&self) -> Node<'_> {
        Node::Ident(self)
    }

    fn facts(&self) -> Name {
        Name::from(self)
    }

    fn opened<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        facts: &Name,
        around: &Around<'_, Contributor::Open>,
    ) -> Contributed<Contributor::Open, Contributor::Part> {
        contributor.on_ident(self, facts, around)
    }

    fn closed<Contributor: Contribute>(
        &self,
        contributor: &Contributor,
        _: &Name,
        inner: Contribution<Contributor::Open, Contributor::Part>,
    ) -> Contributor::Part {
        contributor.off_ident(self, inner)
    }
}
