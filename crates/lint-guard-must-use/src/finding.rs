use std::fmt;

use lint_layout::Name;

pub struct Finding(Name);

impl From<&Name> for Finding {
    fn from(guard: &Name) -> Finding {
        Finding(guard.clone())
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} implements Drop; mark it #[must_use]", self.0)
    }
}
