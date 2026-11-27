use std::ffi::OsString;

use super::{Argument, Utf8};
use crate::cli::command::failure::Failure;

impl Argument for Utf8 {
    type Value = String;

    fn read(word: OsString) -> Result<String, Failure<'static>> {
        word.into_string().map_err(|_unreadable| Failure::Usage)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Flag(&'static str);

impl Flag {
    pub(crate) const fn new(name: &'static str) -> Flag {
        Flag(name)
    }

    pub(super) fn names(self, argument: &OsString) -> Option<()> {
        (argument == self.0).then_some(())
    }
}
