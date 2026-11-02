use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Endpoint, Transition, TypeFact};

use crate::count::Count;
use crate::end::{End, Ends, Node};

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTypes(BTreeMap<Name, BTreeSet<CrateName>>);

impl WorkspaceTypes {
    pub(crate) fn declared(types: &[TypeFact]) -> WorkspaceTypes {
        WorkspaceTypes(types.iter().fold(BTreeMap::new(), |mut names: BTreeMap<Name, BTreeSet<CrateName>>, found| {
            names
                .entry(AsRef::<Name>::as_ref(found).clone())
                .or_default()
                .insert(AsRef::<CrateName>::as_ref(found).clone());
            names
        }))
    }

    pub(crate) fn declared_in(&self, krate: &CrateName) -> Count {
        Count::new(self.0.values().filter(|crates| crates.contains(krate)).count())
    }

    pub(crate) fn ends(&self, transition: &Transition) -> Ends {
        let from: &CrateName = transition.as_ref();
        let takes = transition.takes().iter().map(|taken| self.end(from, taken)).collect();
        Ends::new(takes, self.end(from, transition.gives()))
    }

    fn end(&self, from: &CrateName, endpoint: &Endpoint) -> End {
        endpoint.resolved(
            |name| {
                self.0
                    .get(name)
                    .and_then(|declaring| Node::resolved(declaring, from, name))
                    .map_or(End::Foreign, End::Domain)
            },
            End::Primitive,
            End::Foreign,
        )
    }
}
