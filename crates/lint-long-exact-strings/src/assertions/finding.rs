use std::fmt;

use super::pinning::Pinning;

pub struct Finding(Pinning);

impl From<Pinning> for Finding {
    fn from(pinning: Pinning) -> Finding {
        Finding(pinning)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pinned = match self.0 {
            Pinning::LineBreak => "text with a line break",
            Pinning::Debug | Pinning::Plain => "Debug-formatted text",
        };
        write!(formatter, "assertion pins {pinned}; compare a golden with golden!, or assert on the parts")
    }
}
