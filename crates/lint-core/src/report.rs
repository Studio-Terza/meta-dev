mod headline;

pub use headline::Headline;

use std::fmt;

use crate::boundary::Violation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report<'file> {
    headline: Headline,
    violations: Vec<Violation<'file>>,
}

impl<'file> Report<'file> {
    #[must_use]
    pub fn new(headline: Headline, violations: Vec<Violation<'file>>) -> Report<'file> {
        Report { headline, violations }
    }

    #[must_use]
    pub fn violations(&self) -> &[Violation<'file>] {
        &self.violations
    }

    #[must_use]
    pub fn dirty(self) -> Option<Report<'file>> {
        (!self.violations.is_empty()).then_some(self)
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}:", self.headline)?;
        self.violations.iter().try_for_each(|violation| writeln!(formatter, "{violation}"))
    }
}
