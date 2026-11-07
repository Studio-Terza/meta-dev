mod boundary;

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use grammar::output::field::{Quoted, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;

use crate::Misread;
use crate::briefing::Quoting;
use crate::misread::Briefed;
use boundary::Refusal;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Lapse(Lapsed);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum Lapsed {
    #[error("{0}")]
    Misread(Misread),
    #[error("the run passed its deadline and was killed")]
    Overdue,
    #[error("the directives were refused: {0}")]
    Refused(Refusal),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Lapsing {
    #[name("misread")]
    Misread,
    #[name("overdue")]
    Overdue,
    #[name("refused")]
    Refused,
}

#[derive(Clone, Copy, Debug)]
pub struct Retried<'lapse>(&'lapse Lapse);

impl Lapse {
    #[must_use]
    pub fn overdue() -> Lapse {
        Lapse(Lapsed::Overdue)
    }

    #[must_use]
    pub fn refused(refusal: &dyn Error) -> Lapse {
        Lapse(Lapsed::Refused(Refusal::told(refusal)))
    }

    pub(crate) fn read(fields: &mut Fields) -> Result<Lapse, Unread> {
        match fields.field::<Word<Lapsing>>()? {
            Lapsing::Misread => Misread::read(fields).map(Lapse::from),
            Lapsing::Overdue => Ok(Lapse::overdue()),
            Lapsing::Refused => fields.field::<Quoted<Refusal>>().map(|refusal| Lapse(Lapsed::Refused(refusal))),
        }
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
            Lapsed::Overdue => write!(formatter, "{}", Lapsing::Overdue),
            Lapsed::Refused(refusal) => write!(formatter, "{} {}", Lapsing::Refused, Quoting::from(refusal)),
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
