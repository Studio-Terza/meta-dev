use std::str::FromStr;

use design_model::{NodeId, Unnumbered};

use crate::briefing::Sample;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeRef {
    Stored(NodeId),
    Labelled(Label),
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} cannot label a new node: it is spelled as a node id")]
pub struct Unlabelled(String);

pub trait Bind {
    type Output;

    fn stored(self, id: NodeId) -> Self::Output;

    fn labelled(self, label: &Label) -> Self::Output;
}

impl NodeRef {
    pub fn bound<B: Bind>(&self, binder: B) -> B::Output {
        match self {
            NodeRef::Stored(id) => binder.stored(*id),
            NodeRef::Labelled(label) => binder.labelled(label),
        }
    }
}

impl FromStr for NodeRef {
    type Err = Unnumbered;

    fn from_str(text: &str) -> Result<NodeRef, Unnumbered> {
        text.parse::<NodeId>()
            .map(NodeRef::Stored)
            .or_else(|unnumbered| unnumbered.unspelled().map(|label| NodeRef::Labelled(Label(label))))
    }
}

impl FromStr for Label {
    type Err = Unlabelled;

    fn from_str(text: &str) -> Result<Label, Unlabelled> {
        text.parse::<NodeId>()
            .err()
            .and_then(|unnumbered| unnumbered.unspelled().ok())
            .map(Label)
            .ok_or_else(|| Unlabelled(text.to_owned()))
    }
}

impl From<Sample> for Label {
    fn from(sample: Sample) -> Label {
        Label(sample.to_string())
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
