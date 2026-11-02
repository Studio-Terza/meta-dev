mod share;

use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Site, Violation};
use lint_placement::CrateName;
use type_facts::Transition;

use crate::count::Count;
use crate::end::{Class, Ends, Key};
use crate::workspace_types::WorkspaceTypes;
use share::Share;

#[derive(Debug)]
pub(crate) struct CrateTransitions<'found> {
    name: &'found CrateName,
    first: Option<&'found Site>,
    nodes: Count,
    edges: BTreeSet<Key>,
    classes: Vec<Class>,
}

impl<'found> CrateTransitions<'found> {
    pub(crate) fn measured(
        types: &WorkspaceTypes,
        name: &'found CrateName,
        found: &[&'found Transition],
    ) -> CrateTransitions<'found> {
        let ends: Vec<Ends> = found.iter().map(|transition| types.ends(transition)).collect();
        CrateTransitions {
            name,
            first: found.iter().map(|transition| AsRef::<Site>::as_ref(*transition)).min(),
            nodes: types.declared_in(name),
            edges: ends.iter().flat_map(Ends::edges).collect(),
            classes: ends.iter().map(Ends::class).collect(),
        }
    }

    pub(crate) fn violation(&self) -> Option<Violation> {
        self.first.map(|site| Violation::new(site.clone(), self))
    }

    fn counted(&self, class: Class) -> Count {
        Count::new(self.classes.iter().filter(|found| **found == class).count())
    }
}

impl fmt::Display for CrateTransitions<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = Count::new(self.classes.len());
        write!(
            formatter,
            "crate {}: {} nodes, {} edges, {} transitions, coverage {} of {}, share {}, {} touch a primitive or unit, {} touch a foreign or generic type",
            self.name,
            self.nodes,
            self.edges.len(),
            total,
            self.counted(Class::Covered),
            total,
            Share::between(self.counted(Class::Covered), total),
            self.counted(Class::Primitive),
            self.counted(Class::Foreign)
        )
    }
}
