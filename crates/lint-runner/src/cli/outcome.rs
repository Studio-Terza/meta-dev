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
        if dirty.is_empty() && failures.is_empty() {
            Outcome::Clean
        } else {
            Outcome::Violations { reports: dirty, failures }
        }
    }
}

impl fmt::Display for Outcome<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Clean => formatter.write_str("all lints pass"),
            Outcome::Violations { reports, failures } => {
                reports.iter().try_for_each(|report| write!(formatter, "{report}"))?;
                failures.iter().try_for_each(|failure| writeln!(formatter, "{failure}"))
            }
        }
    }
}
