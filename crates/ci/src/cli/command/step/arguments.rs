use std::ffi::OsString;
use std::vec;

use crate::cli::command::failure::Failure;

#[derive(Debug)]
pub(crate) struct Arguments(vec::IntoIter<OsString>);

impl From<Vec<OsString>> for Arguments {
    fn from(arguments: Vec<OsString>) -> Self {
        Self(arguments.into_iter())
    }
}

impl Arguments {
    pub(crate) fn taken(&mut self) -> Result<OsString, Failure<'static>> {
        self.0.next().ok_or(Failure::Usage)
    }

    pub(crate) fn ended(mut self) -> Result<(), Failure<'static>> {
        self.0.next().map_or(Ok(()), |_| Err(Failure::Usage))
    }

    pub(crate) fn rest(self) -> Vec<OsString> {
        self.0.collect()
    }
}
