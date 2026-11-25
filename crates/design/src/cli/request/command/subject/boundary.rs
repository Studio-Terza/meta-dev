use std::str::FromStr;

use design_model::{JobId, NodeId, Prefix};
use grammar_cited::Cited;
use grammar_output::field::{Bare, Checked, Integer, Word};
use grammar_output::{Fields, Mapped, Parsed};
use named_derive::Named;
use newtype::Wrapping;
use structurer::SegmentRef;

use super::Subject;
use crate::cli::failure::Failure;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Keyed {
    Job,
}

impl FromStr for Subject {
    type Err = Failure;

    fn from_str(word: &str) -> Result<Subject, Failure> {
        let fields = Fields::lexed(word).map_err(|_stray| Failure::Usage)?;
        fields.optional::<Word<Keyed>>().concluded(Keyed::subject)?.ended().map_err(|_unended| Failure::Usage)
    }
}

impl Keyed {
    fn subject(keyed: Option<Keyed>, fields: Fields) -> Result<Parsed<Subject, Fields>, Failure> {
        match keyed {
            Some(Keyed::Job) => fields
                .field::<Integer<_>>()
                .mapped(|job| Subject::Job(JobId::new(job)))
                .map_err(|_unread| Failure::Usage),
            None => fields.optional::<Checked<NodeId>>().concluded(Subject::checked),
        }
    }
}

impl Subject {
    fn checked<Refused>(
        node: Option<Result<NodeId, Refused>>,
        fields: Fields,
    ) -> Result<Parsed<Subject, Fields>, Failure>
    where
        Failure: From<Refused>,
    {
        match node {
            Some(found) => Ok(Parsed::new(Subject::Node(found?), fields)),
            None => fields.optional::<Cited>().concluded(Subject::cited),
        }
    }

    fn cited(cited: Option<Cited>, fields: Fields) -> Result<Parsed<Subject, Fields>, Failure> {
        match cited {
            Some(found) => Ok(Parsed::new(Subject::Segment(SegmentRef::from(found)), fields)),
            None => fields.optional::<Bare<Prefix>>().concluded(Subject::bare),
        }
    }

    fn bare(prefix: Option<Prefix>, fields: Fields) -> Result<Parsed<Subject, Fields>, Failure> {
        prefix.ok_or(Failure::Usage).and_then(Subject::prefixed).map(|subject| Parsed::new(subject, fields))
    }

    fn prefixed(prefix: Prefix) -> Result<Subject, Failure> {
        match prefix.clone().into_inner().parse::<NodeId>() {
            Ok(node) => Ok(Subject::Node(node)),
            Err(unnumbered) => unnumbered.unspelled().map(|_unspelled| Subject::Message(prefix)).map_err(Failure::from),
        }
    }
}
