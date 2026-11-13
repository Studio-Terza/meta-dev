use std::collections::BTreeSet;

use code_model::{LawKind, Model};
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
    pub(crate) fn roundtrips_naming(&self, subject: &Subject<'_>) -> Vec<&'model BTreeSet<Name>> {
        self.model
            .laws()
            .filter(|placed| placed.krate() == subject.krate())
            .map(code_model::Placed::entity)
            .filter(|law| law.kind() == LawKind::Roundtrip)
            .map(code_model::Law::mentions)
            .filter(|mentions| mentions.contains(AsRef::<Name>::as_ref(subject)))
            .collect()
    }
}
