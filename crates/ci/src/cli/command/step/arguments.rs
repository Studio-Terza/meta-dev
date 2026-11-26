mod boundary;

use std::ffi::OsString;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;
use std::vec;

use grammar_output::{Parsed, Source};

use crate::cli::command::failure::Failure;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::measured::Measured;

pub(crate) use boundary::Flag;

#[derive(Debug)]
pub(crate) struct Arguments(iter::Peekable<vec::IntoIter<OsString>>);

pub(crate) trait Argument {
    type Value;

    fn read(word: OsString) -> Result<Self::Value, Failure<'static>>;
}

#[derive(Debug)]
pub(crate) struct Word;

#[derive(Debug)]
pub(crate) struct Location;

#[derive(Debug)]
pub(crate) struct Utf8;

#[derive(Debug)]
pub(crate) struct Recording;

#[derive(Debug)]
pub(crate) struct Journaled;

#[derive(Debug)]
pub(crate) struct Parsing<Read> {
    read: PhantomData<Read>,
}

impl Argument for Recording {
    type Value = Measured;

    fn read(word: OsString) -> Result<Measured, Failure<'static>> {
        Ok(Measured::file(&PathBuf::from(word)))
    }
}

impl Argument for Journaled {
    type Value = Journal;

    fn read(word: OsString) -> Result<Journal, Failure<'static>> {
        Ok(Journal::from(PathBuf::from(word)))
    }
}

impl Argument for Word {
    type Value = OsString;

    fn read(word: OsString) -> Result<OsString, Failure<'static>> {
        Ok(word)
    }
}

impl Argument for Location {
    type Value = PathBuf;

    fn read(word: OsString) -> Result<PathBuf, Failure<'static>> {
        Ok(PathBuf::from(word))
    }
}

impl Argument for Utf8 {
    type Value = String;

    fn read(word: OsString) -> Result<String, Failure<'static>> {
        word.into_string().map_err(|_unreadable| Failure::Usage)
    }
}

impl<Read: FromStr> Argument for Parsing<Read>
where
    Failure<'static>: From<Read::Err>,
{
    type Value = Read;

    fn read(word: OsString) -> Result<Read, Failure<'static>> {
        Ok(word.to_str().ok_or(Failure::Usage)?.parse()?)
    }
}

impl Source for Arguments {
    type Unread = Failure<'static>;
}

pub(crate) trait Ended<Value>: Into<Parsed<Value, Arguments>> {
    fn ended(self) -> Result<Value, Failure<'static>> {
        self.into().concluded(|value, rest| rest.ended().map(|()| value))
    }
}

impl<Value> Ended<Value> for Parsed<Value, Arguments> {}
impl From<Vec<OsString>> for Arguments {
    fn from(arguments: Vec<OsString>) -> Arguments {
        Arguments(arguments.into_iter().peekable())
    }
}

impl Arguments {
    pub(crate) fn flagged<Kind: Argument>(
        mut self,
        flag: Flag,
    ) -> Result<Parsed<Option<Kind::Value>, Arguments>, Failure<'static>> {
        let present = self.0.next_if(|argument| flag.names(argument).is_some());
        let word = present.map(|_flag| self.0.next().ok_or(Failure::Usage)).transpose()?;
        let value = word.map(Kind::read).transpose()?;
        Ok(Parsed::new(value, self))
    }

    pub(crate) fn ended(mut self) -> Result<(), Failure<'static>> {
        self.0.next().map_or(Ok(()), |_| Err(Failure::Usage))
    }

    pub(crate) fn rest(self) -> Vec<OsString> {
        self.0.collect()
    }

    pub(crate) fn field<Kind: Argument>(mut self) -> Result<Parsed<Kind::Value, Arguments>, Failure<'static>> {
        let word = self.0.next().ok_or(Failure::Usage)?;
        Kind::read(word).map(|value| Parsed::new(value, self))
    }

    pub(crate) fn repeated<Kind: Argument<Value = OsString>>(self) -> Parsed<Vec<OsString>, Arguments> {
        Parsed::new(self.0.collect(), Arguments::from(Vec::new()))
    }
}
