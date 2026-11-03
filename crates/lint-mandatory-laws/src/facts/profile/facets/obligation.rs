use std::fmt;

use super::need::Need;
use crate::subject::Subject;
use lint_core::Finding;
use lint_core::Fix;
use lint_core::Site;
use lint_core::Violation;

#[derive(Debug)]
pub(crate) struct Obligation<'facts, 'file> {
    subject: &'facts Subject<'file>,
    site: Site<'file>,
    need: Need<'facts>,
}

impl<'facts, 'file> Obligation<'facts, 'file> {
    pub(crate) fn new(
        subject: &'facts Subject<'file>,
        site: Site<'file>,
        need: Need<'facts>,
    ) -> Obligation<'facts, 'file> {
        Obligation { subject, site, need }
    }
}

impl<'file> From<Obligation<'_, 'file>> for Violation<'file> {
    fn from(obligation: Obligation<'_, 'file>) -> Violation<'file> {
        Violation::new(obligation.site, obligation)
    }
}

impl fmt::Display for Obligation<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` {}", self.subject.owner(), self.need)
    }
}

impl Finding for Obligation<'_, '_> {
    fn fix(&self) -> Fix {
        self.need.fix()
    }
}
