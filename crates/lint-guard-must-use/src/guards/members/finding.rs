use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding<'guard>(&'guard Name);

impl<'guard> From<&'guard Name> for Finding<'guard> {
    fn from(guard: &'guard Name) -> Self {
        Finding(guard)
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} implements Drop", self.0)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("mark it #[must_use]")
    }
}
