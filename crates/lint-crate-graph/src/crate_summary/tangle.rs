use std::fmt;

use lint_core::Fix;
use lint_layout::{Length, Name};
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Tangle(Vec<Name>);

impl Tangle {
    pub const fn items(&self) -> Length {
        Length::new(self.0.len())
    }
}

impl fmt::Display for Tangle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "cycle of {} items: {}", self.0.len(), names.join(" "))
    }
}

impl lint_core::Finding for Tangle {
    fn fix(&self) -> Fix {
        Fix::new("break the cycle with a standard trait, else a generic over an existing trait, else a derive")
    }
}
