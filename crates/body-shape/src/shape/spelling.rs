use std::fmt;

use equivalence::{Equivalence, Erased};
use newtype::{Arbitrary, Display, Newtype};

use super::ShapeToken;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spelling(String);

impl Spelling {
    pub(crate) fn shaped(&self, equivalence: Equivalence) -> ShapeToken<'_> {
        equivalence.erases(Erased::Literals).map_or(ShapeToken::Spelled(self), |_| ShapeToken::Literal)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

impl fmt::Display for Sign {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sign::Plus => Ok(()),
            Sign::Minus => write!(formatter, "-"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spelled {
    sign: Sign,
    spelling: Spelling,
}

impl Spelled {
    pub(crate) fn new(sign: Sign, spelling: Spelling) -> Spelled {
        Spelled { sign, spelling }
    }
}

impl fmt::Display for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.sign, self.spelling)
    }
}
