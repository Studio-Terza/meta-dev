use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::ops::ControlFlow;

use graph::{Event, Trees};

use super::super::boundary::ScopeId;

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
        let forward = |scope: ScopeId| scope.row(successors).into_iter().flatten().copied();
        let finished: Vec<ScopeId> =
            Event::walked(ScopeId::ids(successors), forward).filter_map(Event::finished).collect();
        let edges = ScopeId::ids(successors).flat_map(|from| forward(from).map(move |to| [to, from]));
        let predecessors = edges.fold(BTreeMap::new(), |mut reversed: BTreeMap<ScopeId, Vec<ScopeId>>, [to, from]| {
            reversed.entry(to).or_default().push(from);
            reversed
        });
        let backward = |scope: ScopeId| predecessors.get(&scope).into_iter().flatten().copied();
        let trees: Trees<ScopeId> = Event::walked(finished.iter().rev().copied(), backward).collect();
        Components(trees.into_iter().rev().map(|scopes| Component::from_members(scopes, successors)).collect())
    }

    pub(super) fn settled<State>(&self, state: State, settling: &impl Settling<State>) -> State {
        self.0.iter().fold(state, |settled, component| component.shape.rounds(settled, &component.scopes, settling))
    }
}
