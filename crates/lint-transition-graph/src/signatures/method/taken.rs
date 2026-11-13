use std::fmt;

use crate::transitions::Wrapping;

use crate::end::End;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Taken<'found> {
    end: End<'found>,
    wrapping: Wrapping,
}

impl<'found> Taken<'found> {
    pub(crate) fn new(end: End<'found>, wrapping: Wrapping) -> Taken<'found> {
        Taken { end, wrapping }
    }
}

impl fmt::Display for Taken<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.wrapping {
            Wrapping::Plain => write!(formatter, "{}", self.end),
            Wrapping::Optional => write!(formatter, "Option<{}>", self.end),
            Wrapping::Fallible => write!(formatter, "Result<{}>", self.end),
            Wrapping::Many => write!(formatter, "[{}]", self.end),
        }
    }
}
