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
        write!(formatter, "{}", self.0)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("compare a golden with golden!, or assert on the parts")
    }
}
