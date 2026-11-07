use std::fmt;

use super::{Finding, Fix};
use code_model::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation<'file> {
    site: Site<'file>,
    problem: String,
    fix: Fix,
}

impl<'file> Violation<'file> {
    pub fn new(site: Site<'file>, finding: impl Finding) -> Self {
        Violation { site, problem: finding.to_string(), fix: finding.fix() }
    }

    #[must_use]
    pub fn site(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Violation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}; {}", self.site, self.problem, self.fix)
    }
}
