use std::num::NonZeroI32;
use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Exit {
    Succeeded,
    Failed(Option<NonZeroI32>),
}

impl Exit {
    #[must_use]
    pub fn code(self) -> Option<NonZeroI32> {
        match self {
            Exit::Succeeded => None,
            Exit::Failed(code) => code,
        }
    }
}

impl From<Option<i32>> for Exit {
    fn from(code: Option<i32>) -> Exit {
        match code {
            Some(code) => NonZeroI32::new(code).map_or(Exit::Succeeded, |code| Exit::Failed(Some(code))),
            None => Exit::Failed(None),
        }
    }
}

impl From<ExitStatus> for Exit {
    fn from(status: ExitStatus) -> Exit {
        Exit::from(status.code())
    }
}
