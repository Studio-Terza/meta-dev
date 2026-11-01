use std::num::NonZeroU8;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Success,
    Failure(NonZeroU8),
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> ExitCode {
        match exit {
            Exit::Success => ExitCode::SUCCESS,
            Exit::Failure(code) => ExitCode::from(code.get()),
        }
    }
}
