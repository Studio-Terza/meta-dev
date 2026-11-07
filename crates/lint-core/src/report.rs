use std::fmt;

use crate::boundary::Violation;
use crate::headline::Headline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report<'file> {
    headline: Headline,
    violations: Vec<Violation<'file>>,
}

impl<'file> Report<'file> {
    #[must_use]
    pub const fn new(headline: Headline, violations: Vec<Violation<'file>>) -> Self {
        Report { headline, violations }
    }

    #[must_use]
    pub fn violations(&self) -> &[Violation<'file>] {
        &self.violations
    }

    #[must_use]
    pub fn dirty(self) -> Option<Self> {
        self.violations.first().map(|_| ()).map(|()| self)
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}:", self.headline)?;
        self.violations.iter().try_for_each(|violation| writeln!(formatter, "{violation}"))
    }
}
