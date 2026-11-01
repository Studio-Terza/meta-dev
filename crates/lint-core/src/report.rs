use std::fmt;

use crate::violation::Violation;
use lint_layout::Headline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    headline: Headline,
    violations: Vec<Violation>,
}

impl Report {
    #[must_use]
    pub fn new(headline: Headline, violations: Vec<Violation>) -> Report {
        Report { headline, violations }
    }

    #[must_use]
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}:", self.headline)?;
        self.violations.iter().try_for_each(|violation| writeln!(formatter, "{violation}"))
    }
}
