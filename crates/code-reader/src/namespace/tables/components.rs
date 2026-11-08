use std::ops::ControlFlow;

use super::super::boundary::{Reach, Rounds, ScopeId};

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

    fn visited(self, descent: Descent<'graph>) -> Walk<'graph> {
        let scope = descent.scope;
        let open = self.open.into_iter().chain([scope]).collect();
        let marks = self.marks.opened(scope);
        let opened = Walk { successors: self.successors, marks, open, components: self.components };
        let walk = descent.successors.iter().fold(opened, |walking, next| {
            let reached = walking.reached(*next);
            Walk {
                successors: reached.successors,
                marks: reached.marks.inherited(scope, *next),
                open: reached.open,
                components: reached.components,
            }
        });
        let root = walk.marks.visit(scope).and_then(Visit::root);
        root.into_iter().fold(walk, Walk::closed)
    }

    fn closed(self, root: Reach) -> Walk<'graph> {
        let marks = &self.marks;
        let kept =
            self.open.iter().take_while(|open| marks.visit(**open).map(|visit| visit.order) < Some(root)).count();
        let (open, members) = self.open.split_at(kept);
        let component = Component::from_members(members.to_vec(), self.successors);
        Walk {
            successors: self.successors,
            marks: self.marks.closed(&component),
            open: open.to_vec(),
            components: self.components.into_iter().chain([component]).collect(),
        }
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
    fn rounds<State>(
        self,
        state: State,
        scopes: &[ScopeId],
        rounds: Rounds,
        settling: &impl Settling<State>,
    ) -> ControlFlow<State, State> {
        match self {
            Shape::Acyclic => settling.round(state, scopes),
            Shape::Cyclic => rounds.each().try_fold(state, |settled, ()| settling.round(settled, scopes)),
        }
    }
}

pub(super) trait Settling<State> {
    fn round(&self, state: State, scopes: &[ScopeId]) -> ControlFlow<State, State>;
}

impl Component {
    fn from_members(scopes: Vec<ScopeId>, successors: &[Vec<ScopeId>]) -> Component {
        let inner = scopes.iter().flat_map(|scope| scope.row(successors).into_iter().flatten());
        let shape = inner.clone().find(|next| scopes.contains(next)).map_or(Shape::Acyclic, |_| Shape::Cyclic);
        Component { scopes, shape }
    }

    fn settled<State>(&self, state: State, rounds: Rounds, settling: &impl Settling<State>) -> State {
        let (ControlFlow::Break(settled) | ControlFlow::Continue(settled)) =
            self.shape.rounds(state, &self.scopes, rounds, settling);
        settled
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

    pub(super) fn settled<State>(&self, state: State, rounds: Rounds, settling: &impl Settling<State>) -> State {
        self.0.iter().fold(state, |settled, component| component.settled(settled, rounds, settling))
    }
}
