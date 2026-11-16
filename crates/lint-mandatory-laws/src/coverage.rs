use std::collections::BTreeSet;

use graph::{LawKind, Model};
use lint_layout::Name;

use crate::subject::Subject;

#[derive(Debug)]
pub(crate) struct Coverage<'model> {
    model: &'model Model,
}

impl<'model> From<&'model Model> for Coverage<'model> {
    fn from(model: &'model Model) -> Coverage<'model> {
        Coverage { model }
    }
}

impl<'model> Coverage<'model> {
    pub(crate) fn naming(&self, subject: &Subject<'_>, kind: LawKind) -> Vec<&'model BTreeSet<Name>> {
        self.model
            .laws()
            .filter(|placed| placed.krate() == subject.krate())
            .map(graph::Placed::entity)
            .filter(|law| law.kind() == kind)
            .map(graph::Law::mentions)
            .filter(|mentions| mentions.contains(AsRef::<Name>::as_ref(subject)))
            .collect()
    }
}
