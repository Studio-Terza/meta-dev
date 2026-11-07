use std::num::NonZeroU8;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Success,
    Failure(NonZeroU8),
}

impl Exit {
    #[must_use]
    pub fn excusing(self, excused: &[NonZeroU8]) -> Self {
        match self {
            Self::Failure(code) => excused.iter().find(|excused| **excused == code).map_or(self, |_| Self::Success),
            Self::Success => self,
        }
    }
}

impl Exit {
    pub fn followed_by<Failed>(self, next: impl FnOnce() -> Result<Self, Failed>) -> Result<Self, Failed> {
        match self {
            Self::Success => next(),
            failed @ Self::Failure(_) => Ok(failed),
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        match exit {
            Exit::Success => Self::SUCCESS,
            Exit::Failure(code) => Self::from(code.get()),
        }
    }
}
