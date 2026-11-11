use std::collections::BTreeSet;
use std::iter;
use std::ops::ControlFlow;

use super::super::boundary::{Reach, ScopeId};

#[derive(Clone, Copy, Debug)]
struct Visit {
    order: Reach,
    low: Reach,
}

impl Visit {
    fn root(self) -> Option<Reach> {
        Some(self.order).filter(|order| *order == self.low)
    }
}

#[derive(Clone, Copy, Debug)]
enum Mark {
    Unseen,
    Open(Visit),
    Closed,
}

impl Mark {
    fn open(self) -> Option<Visit> {
        match self {
            Mark::Open(visit) => Some(visit),
            Mark::Unseen | Mark::Closed => None,
        }
    }

    fn lowered(self, reached: Reach) -> Mark {
        match self {
            Mark::Open(visit) => Mark::Open(Visit { order: visit.order, low: visit.low.min(reached) }),
            Mark::Unseen | Mark::Closed => self,
        }
    }
}

#[derive(Debug)]
struct Marks {
    marks: Vec<Mark>,
    visited: Reach,
}

impl Marks {
    fn visit(&self, scope: ScopeId) -> Option<Visit> {
        scope.row(&self.marks).copied().and_then(Mark::open)
    }

    fn opened(mut self, scope: ScopeId) -> Marks {
        let visit = Visit { order: self.visited, low: self.visited };
        scope.row_mut(&mut self.marks).into_iter().for_each(|mark| *mark = Mark::Open(visit));
        Marks { marks: self.marks, visited: self.visited.next() }
    }

    fn inherited(mut self, parent: ScopeId, child: ScopeId) -> Marks {
        let low = self.visit(child).map(|visit| visit.low);
        let row = parent.row_mut(&mut self.marks);
        row.zip(low).into_iter().for_each(|(mark, child_low)| *mark = mark.lowered(child_low));
        self
    }

    fn closed(mut self, component: &Component) -> Marks {
        component
            .scopes
            .iter()
            .for_each(|member| member.row_mut(&mut self.marks).into_iter().for_each(|mark| *mark = Mark::Closed));
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct Descent<'graph> {
    scope: ScopeId,
    successors: &'graph [ScopeId],
}

#[derive(Debug)]
struct Walk<'graph> {
    successors: &'graph [Vec<ScopeId>],
    marks: Marks,
    open: Vec<ScopeId>,
    components: Vec<Component>,
}

impl<'graph> Walk<'graph> {
    fn reached(self, scope: ScopeId) -> Walk<'graph> {
        let unseen = scope.row(&self.marks.marks).filter(|mark| matches!(mark, Mark::Unseen));
        let descent = unseen.and(scope.row(self.successors)).map(|successors| Descent { scope, successors });
        descent.into_iter().fold(self, Walk::visited)
    }

    fn visited(mut self, descent: Descent<'graph>) -> Walk<'graph> {
        let scope = descent.scope;
        self.open.push(scope);
        self.marks = self.marks.opened(scope);
        let walk = descent.successors.iter().fold(self, |walking, next| {
            let reached = walking.reached(*next);
            Walk { marks: reached.marks.inherited(scope, *next), ..reached }
        });
        let root = walk.marks.visit(scope).and_then(Visit::root);
        root.into_iter().fold(walk, Walk::closed)
    }

    fn closed(mut self, root: Reach) -> Walk<'graph> {
        let marks = &self.marks;
        let kept = self.open.partition_point(|open| marks.visit(*open).map(|visit| visit.order) < Some(root));
        let component = Component::from_members(self.open.split_off(kept), self.successors);
        self.marks = self.marks.closed(&component);
        self.components.push(component);
        self
    }
}

#[derive(Debug)]
pub(super) struct Component {
    scopes: Vec<ScopeId>,
    shape: Shape,
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Acyclic,
    Cyclic,
}

impl Shape {
    fn rounds<State>(self, state: State, scopes: &[ScopeId], settling: &impl Settling<State>) -> State {
        match self {
            Shape::Acyclic => settling.written(settling.round(&state, scopes), state),
            Shape::Cyclic => Shape::reached(state, scopes, settling),
        }
    }

    fn reached<State>(state: State, scopes: &[ScopeId], settling: &impl Settling<State>) -> State {
        let mut reached = BTreeSet::new();
        let (ControlFlow::Break(settled) | ControlFlow::Continue(settled)) = iter::repeat(())
            .try_fold(state, |held, ()| settling.stepped(&mut reached, settling.round(&held, scopes), held));
        settled
    }
}

pub(super) trait Settling<State> {
    type Round: Ord + Clone;

    fn round(&self, state: &State, scopes: &[ScopeId]) -> Self::Round;

    fn written(&self, round: Self::Round, state: State) -> State;

    fn stepped(
        &self,
        reached: &mut BTreeSet<Self::Round>,
        round: Self::Round,
        held: State,
    ) -> ControlFlow<State, State> {
        match reached.insert(round.clone()).then_some(()) {
            Some(()) => ControlFlow::Continue(self.written(round, held)),
            None => ControlFlow::Break(held),
        }
    }
}

impl Component {
    fn from_members(scopes: Vec<ScopeId>, successors: &[Vec<ScopeId>]) -> Component {
        let inner = scopes.iter().flat_map(|scope| scope.row(successors).into_iter().flatten());
        let shape = inner.clone().find(|next| scopes.contains(next)).map_or(Shape::Acyclic, |_| Shape::Cyclic);
        Component { scopes, shape }
    }
}

#[derive(Debug)]
pub(super) struct Components(Vec<Component>);

impl Components {
    pub(super) fn ordered(successors: &[Vec<ScopeId>]) -> Components {
        let marks = Marks { marks: vec![Mark::Unseen; successors.len()], visited: Reach::FIRST };
        let walk = Walk { successors, marks, open: Vec::new(), components: Vec::new() };
        let walked = ScopeId::ids(successors).fold(walk, Walk::reached);
        Components(walked.components)
    }

    pub(super) fn settled<State>(&self, state: State, settling: &impl Settling<State>) -> State {
        self.0.iter().fold(state, |settled, component| component.shape.rounds(settled, &component.scopes, settling))
    }
}
