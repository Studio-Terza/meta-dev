use std::fmt;

use lint_core::Fix;
use lint_placement::CrateName;

use super::measures::Measures;

pub(super) struct Finding<'name> {
    name: &'name CrateName,
    measures: Measures,
}

impl<'name> Finding<'name> {
    pub(super) fn new(name: &'name CrateName, measures: Measures) -> Finding<'name> {
        Finding { name, measures }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate {}: {}", self.name, self.measures)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("to break a cycle use a standard trait, else a generic over an existing trait, else a derive")
    }
}
