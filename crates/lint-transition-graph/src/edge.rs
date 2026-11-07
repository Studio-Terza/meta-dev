use std::collections::BTreeMap;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Transition;

use crate::end::Ends;
use crate::form::Form;

#[derive(Debug)]
pub(crate) struct Edge<'found, 'file> {
    transition: Transition<'found, 'file>,
    ends: Ends<'found>,
    fixed_by: Option<&'found Name>,
    form: Form<'found>,
}

impl<'found, 'file> Edge<'found, 'file> {
    pub(crate) fn new(
        transition: Transition<'found, 'file>,
        ends: Ends<'found>,
        fixed_by: Option<&'found Name>,
    ) -> Self {
        let unformed = Edge { transition, ends, fixed_by, form: Form::Transition };
        Edge { form: Form::of_edge(&unformed), ..unformed }
    }

    pub(crate) fn form(&self) -> &Form<'found> {
        &self.form
    }

    pub(crate) fn stored(&self) -> Option<&Self> {
        Some(self).filter(|edge| matches!(edge.form, Form::Store))
    }

    pub(crate) fn ends(&self) -> &Ends<'found> {
        &self.ends
    }

    pub(crate) fn transition(&self) -> &Transition<'found, 'file> {
        &self.transition
    }

    pub(crate) fn site(&self) -> Site<'file> {
        *AsRef::<Site<'file>>::as_ref(&self.transition)
    }

    pub(crate) fn fixed_by(&self) -> Option<&'found Name> {
        self.fixed_by
    }

    pub(crate) fn by_crate<'edge>(
        edges: impl Iterator<Item = &'edge Self>,
    ) -> BTreeMap<&'edge CrateName, Vec<&'edge Self>> {
        edges.fold(BTreeMap::new(), |mut crates, edge| {
            crates.entry(edge.transition().as_ref()).or_default().push(edge);
            crates
        })
    }
}
