use std::num::NonZeroI32;
use std::process::ExitStatus;

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
