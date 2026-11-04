use std::fmt;

use lint_core::Fix;

use super::pinning::Pinning;

#[derive(Debug)]
pub struct Finding(Pinning);

impl From<Pinning> for Finding {
    fn from(pinning: Pinning) -> Finding {
        Finding(pinning)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Pinning::LineBreak => formatter.write_str("assertion pins text with a line break"),
            Pinning::Debug | Pinning::Plain => formatter.write_str("assertion pins Debug-formatted text"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("compare a golden with golden!, or assert on the parts")
    }
}
