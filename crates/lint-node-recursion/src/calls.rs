mod boundary;
mod finding;
mod ignored;
mod key;
mod takes_node;

use std::collections::{BTreeMap, BTreeSet};

use graph::{Function, Model, Placed};
use lint_core::Violation;
use lint_layout::{Name, Word};

use crate::calls::ignored::Ignored;
use boundary::Callable;
use finding::Recursive;
use invariants::Ignoring;
use key::Key;
use takes_node::TakesNode;

#[derive(Debug)]
struct Frontier {
    pending: BTreeSet<Callable>,
    seen: BTreeSet<Callable>,
}

impl Frontier {
    fn next(&self, graph: &CallGraph<'_>) -> Option<Frontier> {
        let seen: BTreeSet<Callable> = self.seen.union(&self.pending).copied().collect();
        let reached: BTreeSet<Callable> = self.pending.iter().flat_map(|callable| graph.callees(*callable)).collect();
        let pending: BTreeSet<Callable> = reached.difference(&seen).copied().collect();
        Some(Frontier { pending, seen }).filter(|next| next.pending.first().is_some())
    }
}

#[derive(Debug)]
pub struct CallGraph<'model> {
    functions: Vec<Placed<'model, &'model Function>>,
    keyed: BTreeMap<Key<'model>, Vec<Callable>>,
}

impl<'model> CallGraph<'model> {
    const SYNTAX: Word<'static> = Word::new("syntax");

    pub fn of_model(model: &'model Model) -> CallGraph<'model> {
        let functions: Vec<Placed<'model, &'model Function>> = model.functions().collect();
        let listed = Callable::every(&functions).filter_map(|callable| Some((callable, callable.picked(&functions)?)));
        let keyed =
            listed.fold(BTreeMap::new(), |mut keyed: BTreeMap<Key<'model>, Vec<Callable>>, (callable, placed)| {
                keyed.entry(Key::from(*placed)).or_default().push(callable);
                keyed
            });
        CallGraph { functions, keyed }
    }

    pub fn recursive_over_nodes(&self) -> Vec<Violation<'model>> {
        let syntax = Name::from(Self::SYNTAX);
        Callable::every(&self.functions)
            .filter_map(|callable| Some((callable, callable.picked(&self.functions)?)))
            .filter(|(_, placed)| *placed.krate() != syntax)
            .filter(|(_, placed)| TakesNode::function(placed.entity()).is_some())
            .filter(|(callable, _)| self.reaches_itself(*callable).is_some())
            .map(|(_, placed)| Violation::new(placed.site(), Recursive::from(AsRef::<Name>::as_ref(placed.entity()))))
            .collect()
    }

    fn callees(&self, callable: Callable) -> Vec<Callable> {
        callable.picked(&self.functions).map_or_default(|placed| {
            let caller = Key::from(*placed);
            let invocations = placed.entity().definition().map_or(&[][..], graph::Definition::invocations);
            invocations
                .iter()
                .filter_map(|invocation| caller.called(invocation))
                .filter_map(|key| self.keyed.get(&key))
                .filter_map(|candidates| <&[Callable; 1]>::try_from(candidates.as_slice()).ignoring(Ignored::Ambiguous))
                .map(|[only]| *only)
                .collect()
        })
    }

    fn reaches_itself(&self, start: Callable) -> Option<()> {
        let first = Frontier { pending: self.callees(start).into_iter().collect(), seen: BTreeSet::new() };
        let mut frontiers = std::iter::successors(Some(first), |frontier| frontier.next(self));
        frontiers.find(|frontier| frontier.pending.contains(&start)).map(|_| ())
    }
}
