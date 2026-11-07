use std::fmt::{self, Write};
use std::iter;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Line(String);

impl AsRef<str> for Line {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let leads = iter::once("").chain(iter::repeat("'"));
        formatter.write_char('"')?;
        self.0.split('\'').map(Escaped).zip(leads).try_for_each(|(piece, lead)| write!(formatter, "{lead}{piece}"))?;
        formatter.write_char('"')
    }
}

struct Escaped<'line>(&'line str);

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.chars().map(char::escape_debug).try_for_each(|escaped| write!(formatter, "{escaped}"))
    }
}
