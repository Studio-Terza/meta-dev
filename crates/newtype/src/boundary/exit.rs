use std::num::NonZeroI32;
use std::process::{ExitCode, ExitStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Exit {
    Succeeded,
    Failed(NonZeroI32),
    Signalled,
}

impl From<Option<i32>> for Exit {
    fn from(code: Option<i32>) -> Exit {
        match code {
            Some(code) => NonZeroI32::new(code).map_or(Exit::Succeeded, Exit::Failed),
            None => Exit::Signalled,
        }
    }
}

impl From<ExitStatus> for Exit {
    fn from(status: ExitStatus) -> Exit {
        Exit::from(status.code())
    }
}

impl Exit {
    pub const FAILURE: Exit = Exit::Failed(NonZeroI32::new(1).unwrap());
    pub const SIGNALLED_CODE: u8 = 128;

    #[must_use]
    pub fn excusing(self, excused: &[NonZeroI32]) -> Exit {
        excused.iter().find(|code| Exit::Failed(**code) == self).map_or(self, |_| Exit::Succeeded)
    }

    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Exit::Succeeded => 0,
            Exit::Failed(code) => Exit::clamped(code),
            Exit::Signalled => Exit::SIGNALLED_CODE,
        }
    }

    fn clamped(code: NonZeroI32) -> u8 {
        u8::try_from(code.get().clamp(1, i32::from(u8::MAX))).unwrap_or(u8::MAX)
    }

    pub fn followed_by<Failed>(self, next: impl FnOnce() -> Result<Exit, Failed>) -> Result<Exit, Failed> {
        match self {
            Exit::Succeeded => next(),
            stopped => Ok(stopped),
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> ExitCode {
        ExitCode::from(exit.code())
    }
}
