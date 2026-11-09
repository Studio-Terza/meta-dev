mod boundary;

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use grammar_output::field::{Quoted, Word};
use grammar_output::{Fields, Unread};
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
    #[name("misread")]
    Misread,
    #[name("refused")]
    Refused,
    #[name("uncovered")]
    Uncovered,
}

#[derive(Clone, Copy, Debug)]
pub struct Retried<'lapse>(&'lapse Lapse);

impl Lapse {
    #[must_use]
    pub fn refused(refusal: &dyn Error) -> Lapse {
        Lapse(Lapsed::Refused(Reason::told(refusal)))
    }

    pub(crate) fn read(fields: &mut Fields) -> Result<Lapse, Unread> {
        match fields.field::<Word<Lapsing>>()? {
            Lapsing::Misread => Misread::read(fields).map(Lapse::from),
            Lapsing::Refused => fields.field::<Quoted<Reason>>().map(|refusal| Lapse(Lapsed::Refused(refusal))),
            Lapsing::Uncovered => Uncovered::try_from(fields).map(Lapse::from),
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
        let mut fields = Fields::lexed(text)?;
        let lapse = Lapse::read(&mut fields)?;
        fields.ended(lapse)
    }
}
