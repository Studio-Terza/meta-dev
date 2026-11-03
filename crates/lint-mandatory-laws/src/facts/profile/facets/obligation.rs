use std::fmt;

use super::Need;
use crate::subject::Subject;
use lint_core::Finding;
use lint_core::Fix;

#[derive(Debug)]
pub(crate) struct Obligation<'facts, 'file> {
    subject: &'facts Subject<'file>,
    need: Need<'facts>,
}

impl<'facts, 'file> Obligation<'facts, 'file> {
    pub(crate) fn new(subject: &'facts Subject<'file>, need: Need<'facts>) -> Obligation<'facts, 'file> {
        Obligation { subject, need }
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
