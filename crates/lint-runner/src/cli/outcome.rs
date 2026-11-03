use std::fmt;

use lint_core::Report;
use lint_layout::Failure;

#[derive(Debug)]
pub enum Outcome<'file> {
    Clean,
    Violations { reports: Vec<Report<'file>>, failures: &'file [Failure] },
}

impl<'file> Outcome<'file> {
    #[must_use]
    pub fn from_reports(reports: Vec<Report<'file>>, failures: &'file [Failure]) -> Outcome<'file> {
        let dirty: Vec<Report<'file>> = reports.into_iter().filter_map(Report::dirty).collect();
        dirty
            .first()
            .map(|_| ())
            .or(failures.first().map(|_| ()))
            .map_or(Outcome::Clean, |()| Outcome::Violations { reports: dirty, failures })
    }

    pub(crate) fn violations(&self) -> Option<&Outcome<'file>> {
        match self {
            Outcome::Clean => None,
            Outcome::Violations { .. } => Some(self),
        }
    }

    fn listed(reports: &[Report<'_>], failures: &[Failure], formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        reports.iter().try_for_each(|report| write!(formatter, "{report}"))?;
        failures.iter().try_for_each(|failure| writeln!(formatter, "{failure}"))
    }
}

impl fmt::Display for Outcome<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Clean => formatter.write_str("all lints pass"),
            Outcome::Violations { reports, failures } => Self::listed(reports, failures, formatter),
        }
    }
}
