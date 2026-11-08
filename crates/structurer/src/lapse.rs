mod boundary;

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use dispatch_ports::{Errored, Unreadable};
use grammar::output::field::{Quoted, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;

use crate::briefing::Quoting;
use crate::misread::Briefed;
use crate::owed::Listed;
use crate::{Misread, Owed, Uncovered};
use boundary::Reason;

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
    Refused(Reason),
    #[error("the agent's output was unreadable: {0}")]
    Unreadable(Reason),
    #[error("the agent's run ended in error: {0}")]
    Errored(Reason),
    #[error("{0}")]
    Uncovered(Uncovered),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Lapsing {
    #[name("misread")]
    Misread,
    #[name("overdue")]
    Overdue,
    #[name("refused")]
    Refused,
    #[name("unreadable")]
    Unreadable,
    #[name("errored")]
    Errored,
    #[name("uncovered")]
    Uncovered,
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
        Lapse(Lapsed::Refused(Reason::told(refusal)))
    }

    pub(crate) fn read(fields: &mut Fields) -> Result<Lapse, Unread> {
        match fields.field::<Word<Lapsing>>()? {
            Lapsing::Misread => Misread::read(fields).map(Lapse::from),
            Lapsing::Overdue => Ok(Lapse::overdue()),
            Lapsing::Refused => fields.field::<Quoted<Reason>>().map(|refusal| Lapse(Lapsed::Refused(refusal))),
            Lapsing::Unreadable => {
                fields.field::<Quoted<Reason>>().map(|unreadable| Lapse(Lapsed::Unreadable(unreadable)))
            }
            Lapsing::Errored => fields.field::<Quoted<Reason>>().map(|errored| Lapse(Lapsed::Errored(errored))),
            Lapsing::Uncovered => Owed::from(fields).paid(&BTreeSet::new()).err().map(Lapse::from).ok_or(Unread::Ended),
        }
    }
}

impl From<&Unreadable> for Lapse {
    fn from(unreadable: &Unreadable) -> Lapse {
        Lapse(Lapsed::Unreadable(Reason::told(unreadable)))
    }
}

impl From<&Errored> for Lapse {
    fn from(errored: &Errored) -> Lapse {
        Lapse(Lapsed::Errored(Reason::told(errored)))
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
            Lapsed::Overdue => write!(formatter, "{}", Lapsing::Overdue),
            Lapsed::Refused(refusal) => write!(formatter, "{} {}", Lapsing::Refused, Quoting::from(refusal)),
            Lapsed::Unreadable(unreadable) => {
                write!(formatter, "{} {}", Lapsing::Unreadable, Quoting::from(unreadable))
            }
            Lapsed::Errored(errored) => write!(formatter, "{} {}", Lapsing::Errored, Quoting::from(errored)),
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
