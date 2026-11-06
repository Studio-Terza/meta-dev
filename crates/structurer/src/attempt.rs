use std::error::Error;
use std::fmt;

use dispatch_ports::Reply;

use crate::{Directives, Misread};

#[derive(Debug, thiserror::Error)]
pub enum Lapse {
    #[error("the reply is not directives: {0}")]
    Misread(#[from] Misread),
    #[error("the run passed its deadline and was killed")]
    Overdue,
    #[error("the directives were refused: {0}")]
    Refused(#[source] Box<dyn Error + Send + Sync>),
}

#[derive(Debug)]
pub enum Attempt {
    First,
    Retry(Lapse),
}

#[derive(Debug)]
pub enum Reading {
    Parsed { directives: Directives, attempt: Attempt },
    Again(Attempt),
    Flailed(Lapse),
}

impl Attempt {
    const RETRY: &'static str = "Your previous reply could not be used";

    #[must_use]
    pub fn lapsed(self, lapse: Lapse) -> Reading {
        match self {
            Attempt::First => Reading::Again(Attempt::Retry(lapse)),
            Attempt::Retry(_earlier) => Reading::Flailed(lapse),
        }
    }

    #[must_use]
    pub fn read(self, reply: &Reply) -> Reading {
        match reply.as_ref().parse::<Directives>() {
            Ok(directives) => Reading::Parsed { directives, attempt: self },
            Err(misread) => self.lapsed(Lapse::from(misread)),
        }
    }
}

impl fmt::Display for Attempt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attempt::First => Ok(()),
            Attempt::Retry(lapse) => {
                write!(formatter, "\n{}: {lapse}. Reply again, following the grammar exactly.\n", Self::RETRY)
            }
        }
    }
}
