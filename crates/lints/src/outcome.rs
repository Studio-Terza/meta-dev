use std::fmt;

use lint_core::{Failure, Report};

#[derive(Debug)]
pub enum Outcome {
    Clean,
    Violations { reports: Vec<Report>, failures: Vec<Failure> },
}

impl Outcome {
    #[must_use]
    pub fn from_reports(reports: Vec<Report>, failures: Vec<Failure>) -> Outcome {
        let dirty: Vec<Report> = reports.into_iter().filter(|report| !report.is_clean()).collect();
        if dirty.is_empty() && failures.is_empty() {
            Outcome::Clean
        } else {
            Outcome::Violations { reports: dirty, failures }
        }
    }
}

impl fmt::Display for Outcome {
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
