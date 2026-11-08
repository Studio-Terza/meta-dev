use std::str::FromStr;

use design_model::{NodeId, Prefix};
use grammar::cited::Cited;
use grammar::output::Fields;
use grammar::output::field::{Bare, Checked};
use newtype::Wrapping;
use structurer::SegmentRef;

use super::Subject;
use crate::cli::failure::Failure;

impl FromStr for Subject {
    type Err = Failure;

    fn from_str(word: &str) -> Result<Subject, Failure> {
        let mut fields = Fields::lexed(word).map_err(|_stray| Failure::Usage)?;
        let subject = fields
            .optional::<Checked<NodeId>>()
            .map(|node| Ok(Subject::Node(node?)))
            .or_else(|| fields.optional::<Cited>().map(|cited| Ok(Subject::Segment(SegmentRef::from(&cited)))))
            .or_else(|| fields.optional::<Bare<Prefix>>().map(Subject::prefixed))
            .ok_or(Failure::Usage)??;
        fields.ended(subject).map_err(|_unended| Failure::Usage)
    }
}

impl Subject {
    fn prefixed(prefix: Prefix) -> Result<Subject, Failure> {
        match prefix.clone().into_inner().parse::<NodeId>() {
            Ok(node) => Ok(Subject::Node(node)),
            Err(unnumbered) => unnumbered.unspelled().map(|_unspelled| Subject::Message(prefix)).map_err(Failure::from),
        }
    }
}
