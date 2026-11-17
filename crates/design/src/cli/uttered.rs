use console_ports::Shown;
use port_core::{ReleaseFailures, Snapshot};

use super::failure::Failure;
use crate::Outcome;

#[derive(Debug)]
pub struct Uttered {
    shown: Shown,
    ended: Result<Outcome, Failure>,
}

impl Uttered {
    pub(crate) fn new(shown: Shown, ended: Result<Outcome, Failure>) -> Uttered {
        Uttered { shown, ended }
    }

    pub(crate) fn silent(ended: Result<Outcome, Failure>) -> Uttered {
        Uttered::new(Shown::default(), ended)
    }

    pub(crate) fn kept<Kept>(self, bookkept: Result<Kept, Failure>) -> Uttered {
        Uttered { ended: self.ended.and_then(|outcome| bookkept.map(|_kept| outcome)), ..self }
    }

    pub(crate) fn settled(self, released: Result<Snapshot, ReleaseFailures>) -> Uttered {
        Uttered { ended: Failure::settled(self.ended, released), ..self }
    }

    pub fn ended(self) -> Result<Outcome, Failure> {
        self.ended
    }
}

impl AsRef<Shown> for Uttered {
    fn as_ref(&self) -> &Shown {
        &self.shown
    }
}
