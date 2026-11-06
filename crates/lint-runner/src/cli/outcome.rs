use std::fmt;

use lint_layout::Failure;

use super::boundary::Rendered;

#[derive(Debug)]
pub enum Outcome<'file> {
    Clean,
    Violations { reports: Vec<Rendered>, failures: &'file [Failure] },
}

impl<'file> Outcome<'file> {
    #[must_use]
    pub fn from_rendered(reports: Vec<Rendered>, failures: &'file [Failure]) -> Outcome<'file> {
        reports
            .is_empty()
            .then_some(failures)
            .filter(|failures| failures.is_empty())
            .map_or(Outcome::Violations { reports, failures }, |_| Outcome::Clean)
    }

    pub(crate) fn violations(&self) -> Option<&Outcome<'file>> {
        match self {
            Outcome::Clean => None,
            Outcome::Violations { .. } => Some(self),
        }
    }
}

impl fmt::Display for Outcome<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Clean => formatter.write_str("all lints pass"),
            Outcome::Violations { reports, failures } => reports
                .iter()
                .try_for_each(|report| write!(formatter, "{report}"))
                .and(failures.iter().try_for_each(|failure| writeln!(formatter, "{failure}"))),
        }
    }
}
