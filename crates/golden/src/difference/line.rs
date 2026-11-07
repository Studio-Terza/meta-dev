use std::char::EscapeDebug;
use std::cmp::Ordering;
use std::fmt::{self, Write};

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
        formatter.write_char('"')?;
        self.0.chars().map(Shown::from).try_for_each(|shown| write!(formatter, "{shown}"))?;
        formatter.write_char('"')
    }
}

enum Shown {
    Apostrophe,
    Escaped(EscapeDebug),
}

impl Shown {
    const APOSTROPHE: char = '\'';
}

impl From<char> for Shown {
    fn from(character: char) -> Shown {
        match character.cmp(&Shown::APOSTROPHE) {
            Ordering::Equal => Shown::Apostrophe,
            Ordering::Less | Ordering::Greater => Shown::Escaped(character.escape_debug()),
        }
    }
}

impl fmt::Display for Shown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shown::Apostrophe => formatter.write_char(Self::APOSTROPHE),
            Shown::Escaped(escaped) => write!(formatter, "{escaped}"),
        }
    }
}
