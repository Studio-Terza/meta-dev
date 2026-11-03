use std::fmt;

use super::need::Need;
use crate::subject::Subject;
use lint_core::Finding;
use lint_core::Fix;
use lint_core::Site;
use lint_core::Violation;

#[derive(Debug)]
pub(crate) struct Obligation<'file> {
    subject: Subject,
    site: Site<'file>,
    need: Need,
}

impl<'file> Obligation<'file> {
    pub(crate) fn new(subject: Subject, site: Site<'file>, need: Need) -> Obligation<'file> {
        Obligation { subject, site, need }
    }
}

impl<'file> From<Obligation<'file>> for Violation<'file> {
    fn from(obligation: Obligation<'file>) -> Violation<'file> {
        Violation::new(obligation.site, obligation)
    }
}

impl fmt::Display for Obligation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` {}", self.subject.owner(), self.need)
    }
}

impl Finding for Obligation<'_> {
    fn fix(&self) -> Fix {
        self.need.fix()
    }
}
