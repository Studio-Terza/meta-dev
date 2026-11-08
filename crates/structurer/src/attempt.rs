use dispatch_ports::Reply;
use newtype::Newtype;

use crate::{Briefing, Directives, Lapse};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attempt {
    First,
    Retry(Lapse),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Parsed { directives: Directives, attempt: Attempt },
    Again(Attempt),
    Flailed(Lapse),
}

impl Attempt {
    #[must_use]
    pub fn lapsed(self, lapse: Lapse) -> Reading {
        match self {
            Attempt::First => Reading::Again(Attempt::Retry(lapse)),
            Attempt::Retry(_earlier) => Reading::Flailed(lapse),
        }
    }

    #[must_use]
    pub fn read(self, reply: &Reply) -> Reading {
        match reply.clone().into_inner().parse::<Directives>() {
            Ok(directives) => Reading::Parsed { directives, attempt: self },
            Err(misread) => self.lapsed(Lapse::from(misread)),
        }
    }
}

impl Attempt {
    #[must_use]
    pub fn lapse(&self) -> Option<&Lapse> {
        match self {
            Attempt::First => None,
            Attempt::Retry(lapse) => Some(lapse),
        }
    }

    pub(crate) fn briefed(&self, first: Briefing) -> Briefing {
        match self {
            Attempt::First => first,
            Attempt::Retry(lapse) => first.retried(lapse),
        }
    }
}
