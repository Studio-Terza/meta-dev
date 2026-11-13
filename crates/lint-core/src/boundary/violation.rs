use std::fmt;
use std::path::Path;

use super::{Finding, Fix};
use graph::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation<'file> {
    site: Site<'file>,
    problem: String,
    fix: Fix,
}

impl<'file> Violation<'file> {
    pub fn new(site: Site<'file>, finding: impl Finding) -> Violation<'file> {
        Violation { site, problem: finding.to_string(), fix: finding.fix() }
    }

    #[must_use]
    pub fn site(&self) -> &Site<'file> {
        &self.site
    }

    #[must_use]
    pub fn moved(self, path: &Path) -> Violation<'_> {
        Violation { site: self.site.moved(path), problem: self.problem, fix: self.fix }
    }
}

impl fmt::Display for Violation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}; {}", self.site, self.problem, self.fix)
    }
}
