use lint_core::Site;
use transition_facts::Transition;

use super::label::Label;
use crate::edge::Edge;
use crate::end::Step;

#[derive(Debug)]
pub(crate) struct Arc<'found> {
    step: Step,
    label: Label,
    transition: &'found Transition,
}

impl<'found> Arc<'found> {
    pub(super) fn of_edge(edge: &Edge<'found>) -> Vec<Arc<'found>> {
        let label = Label::read(edge.role(), edge.transition());
        edge.ends()
            .steps()
            .into_iter()
            .map(|step| Arc { step, label: label.clone(), transition: edge.transition() })
            .collect()
    }

    pub(crate) fn transition(&self) -> &'found Transition {
        self.transition
    }
}

impl AsRef<Step> for Arc<'_> {
    fn as_ref(&self) -> &Step {
        &self.step
    }
}

impl AsRef<Label> for Arc<'_> {
    fn as_ref(&self) -> &Label {
        &self.label
    }
}

impl AsRef<Site> for Arc<'_> {
    fn as_ref(&self) -> &Site {
        self.transition.as_ref()
    }
}
