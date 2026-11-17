mod boundary;

use std::ffi::OsString;
use std::iter;
use std::path::PathBuf;
use std::vec;

use crate::cli::command::failure::Failure;

pub(crate) use boundary::Flag;

#[derive(Debug)]
pub(crate) struct Arguments(iter::Peekable<vec::IntoIter<OsString>>);

#[derive(Debug)]
pub(crate) struct Flagged {
    value: Option<OsString>,
    rest: Arguments,
}

impl Flagged {
    pub(crate) fn path(&self) -> Option<PathBuf> {
        self.value.as_ref().map(PathBuf::from)
    }

    pub(crate) fn following(self) -> Arguments {
        self.rest
    }
}

impl From<Vec<OsString>> for Arguments {
    fn from(arguments: Vec<OsString>) -> Arguments {
        Arguments(arguments.into_iter().peekable())
    }
}

impl Arguments {
    pub(crate) fn taken(&mut self) -> Result<OsString, Failure<'static>> {
        self.0.next().ok_or(Failure::Usage)
    }

    pub(crate) fn flagged(mut self, flag: Flag) -> Result<Flagged, Failure<'static>> {
        let present = self.0.next_if(|argument| flag.names(argument).is_some());
        let value = present.map(|_flag| self.taken()).transpose()?;
        Ok(Flagged { value, rest: self })
    }

    pub(crate) fn ended(mut self) -> Result<(), Failure<'static>> {
        self.0.next().map_or(Ok(()), |_| Err(Failure::Usage))
    }

    pub(crate) fn rest(self) -> Vec<OsString> {
        self.0.collect()
    }
}
