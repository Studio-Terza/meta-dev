use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding(Name);

impl From<&Name> for Finding {
    fn from(guard: &Name) -> Finding {
        Finding(guard.clone())
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} implements Drop", self.0)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("mark it #[must_use]")
    }
}
