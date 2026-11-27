use std::num::NonZeroU8;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Success,
    Failure(NonZeroU8),
}

impl Exit {
    #[must_use]
    pub fn excusing(self, excused: &[NonZeroU8]) -> Exit {
        match self {
            Exit::Failure(code) => excused.iter().find(|allowed| **allowed == code).map_or(self, |_| Exit::Success),
            Exit::Success => self,
        }
    }
}

impl Exit {
    pub fn followed_by<Failed>(self, next: impl FnOnce() -> Result<Exit, Failed>) -> Result<Exit, Failed> {
        match self {
            Exit::Success => next(),
            failed @ Exit::Failure(_) => Ok(failed),
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> ExitCode {
        match exit {
            Exit::Success => ExitCode::SUCCESS,
            Exit::Failure(code) => ExitCode::from(code.get()),
        }
    }
}
