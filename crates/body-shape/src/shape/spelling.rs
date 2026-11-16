use std::fmt;

use super::Code;

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
    code: Code,
}

impl Spelled {
    pub(crate) fn new(sign: Sign, code: Code) -> Spelled {
        Spelled { sign, code }
    }
}

impl fmt::Display for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.sign, self.code)
    }
}
