use std::fmt::{self, Write};

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Line(String);

impl Line {
    fn escaped(formatter: &mut fmt::Formatter<'_>, character: char) -> fmt::Result {
        match character {
            '\'' => formatter.write_char(character),
            _ => write!(formatter, "{}", character.escape_debug()),
        }
    }
}

impl AsRef<str> for Line {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_char('"')?;
        self.0.chars().try_for_each(|character| Self::escaped(formatter, character))?;
        formatter.write_char('"')
    }
}
