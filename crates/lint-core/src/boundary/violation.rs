use std::fmt;

use crate::site::Site;
use lint_layout::{Finding, Fix};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    site: Site,
    problem: String,
    fix: Fix,
}

impl Violation {
    pub fn new(site: Site, finding: impl Finding) -> Violation {
        Violation { site, problem: finding.to_string(), fix: finding.fix() }
    }

    #[must_use]
    pub fn site(&self) -> &Site {
        &self.site
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}; {}", self.site, self.problem, self.fix)
    }
}
