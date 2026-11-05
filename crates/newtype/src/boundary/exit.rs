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

impl From<Exit> for u8 {
    fn from(exit: Exit) -> u8 {
        match exit {
            Exit::Succeeded => 0,
            Exit::Failed(code) => u8::try_from(code.get()).unwrap_or(1),
            Exit::Signalled => 1,
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> ExitCode {
        ExitCode::from(u8::from(exit))
    }
}
