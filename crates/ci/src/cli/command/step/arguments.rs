mod boundary;

use std::ffi::OsString;
use std::iter;
use std::vec;

use crate::cli::command::failure::Failure;

pub(crate) use boundary::Flag;

#[derive(Debug)]
pub(crate) struct Arguments(iter::Peekable<vec::IntoIter<OsString>>);

impl From<Vec<OsString>> for Arguments {
    fn from(arguments: Vec<OsString>) -> Arguments {
        Arguments(arguments.into_iter().peekable())
    }
}

impl Arguments {
    pub(crate) fn taken(&mut self) -> Result<OsString, Failure<'static>> {
        self.0.next().ok_or(Failure::Usage)
    }

    pub(crate) fn flagged(&mut self, flag: Flag) -> Result<Option<OsString>, Failure<'static>> {
        self.0.next_if(|argument| flag.names(argument).is_some()).map(|_flag| self.taken()).transpose()
    }

    pub(crate) fn ended(mut self) -> Result<(), Failure<'static>> {
        self.0.next().map_or(Ok(()), |_| Err(Failure::Usage))
    }

    pub(crate) fn rest(self) -> Vec<OsString> {
        self.0.collect()
    }
}
