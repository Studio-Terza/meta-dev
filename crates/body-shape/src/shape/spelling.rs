use std::fmt;

use equivalence::{Equivalence, Erased};
use lint_layout::Name;
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
            Self::Plus => Ok(()),
            Self::Minus => write!(formatter, "-"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Spoken {
    Literal(Spelling),
    Word(Name),
}

impl fmt::Display for Spoken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(spelling) => write!(formatter, "{spelling}"),
            Self::Word(name) => write!(formatter, "{name}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spelled {
    sign: Sign,
    spoken: Spoken,
}

impl Spelled {
    pub(crate) fn new(sign: Sign, spoken: Spoken) -> Self {
        Self { sign, spoken }
    }
}

impl fmt::Display for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.sign, self.spoken)
    }
}
