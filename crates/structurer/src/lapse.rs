mod boundary;

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use grammar_output::field::{Quoted, Word};
use grammar_output::{Fields, Mapped, Parsed, Record, Unread};
use named_derive::Named;

use crate::briefing::Quoting;
use crate::misread::Briefed;
use crate::uncovered::Listed;
use crate::{Misread, Uncovered};
use boundary::Reason;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Lapse(Lapsed);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum Lapsed {
    #[error("{0}")]
    Misread(Misread),
    #[error("the directives were refused: {0}")]
    Refused(Reason),
    #[error("{0}")]
    Uncovered(Uncovered),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Lapsing {
    Misread,
    Refused,
    Uncovered,
}

#[derive(Clone, Copy, Debug)]
pub struct Retried<'lapse>(&'lapse Lapse);

impl Lapse {
    #[must_use]
    pub fn refused(refusal: &dyn Error) -> Lapse {
        Lapse(Lapsed::Refused(Reason::told(refusal)))
    }
}

impl Record for Lapse {
    fn read(fields: Fields) -> Result<Parsed<Lapse, Fields>, Unread> {
        fields.field::<Word<Lapsing>>()?.concluded(Lapsing::lapse)
    }
}

impl Lapsing {
    fn lapse(self, fields: Fields) -> Result<Parsed<Lapse, Fields>, Unread> {
        match self {
            Lapsing::Misread => fields.record::<Misread>().mapped(Lapse::from),
            Lapsing::Refused => fields.field::<Quoted<Reason>>().mapped(|reason| Lapse(Lapsed::Refused(reason))),
            Lapsing::Uncovered => fields.record::<Uncovered>().mapped(Lapse::from),
        }
    }
}

impl From<Uncovered> for Lapse {
    fn from(uncovered: Uncovered) -> Lapse {
        Lapse(Lapsed::Uncovered(uncovered))
    }
}

impl From<Misread> for Lapse {
    fn from(misread: Misread) -> Lapse {
        Lapse(Lapsed::Misread(misread))
    }
}

impl fmt::Display for Retried<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.0 {
            Lapsed::Misread(misread) => write!(formatter, "{} {}", Lapsing::Misread, Briefed::from(misread)),
            Lapsed::Refused(refusal) => write!(formatter, "{} {}", Lapsing::Refused, Quoting::from(refusal)),
            Lapsed::Uncovered(uncovered) => write!(formatter, "{}{}", Lapsing::Uncovered, Listed::from(uncovered)),
        }
    }
}

impl<'lapse> From<&'lapse Lapse> for Retried<'lapse> {
    fn from(lapse: &'lapse Lapse) -> Retried<'lapse> {
        Retried(lapse)
    }
}

impl FromStr for Lapse {
    type Err = Unread;

    fn from_str(text: &str) -> Result<Lapse, Unread> {
        Fields::lexed(text)?.record::<Lapse>()?.ended()
    }
}
