use std::fmt;

use lint_core::Report;

#[derive(Debug)]
pub enum Outcome {
    Clean,
    Violations(Vec<Report>),
}

impl Outcome {
    #[must_use]
    pub fn from_reports(reports: Vec<Report>) -> Outcome {
        let dirty: Vec<Report> = reports.into_iter().filter(|report| !report.is_clean()).collect();
        if dirty.is_empty() { Outcome::Clean } else { Outcome::Violations(dirty) }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Clean => formatter.write_str("all lints pass"),
            Outcome::Violations(reports) => reports.iter().try_for_each(|report| write!(formatter, "{report}")),
        }
    }
}
