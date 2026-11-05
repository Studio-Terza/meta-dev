mod matching;

use std::fmt;

use lint_core::{Finding, Fix, Violation};
use test_facts::{Check, Test};

use matching::Matching;

#[derive(Debug)]
pub(crate) struct Lost<'tests, 'file> {
    test: &'tests Test<'file>,
    check: Check,
}

impl<'tests, 'file> Lost<'tests, 'file> {
    pub(crate) fn unmatched(before: &[&'tests Test<'file>], after: &[Check]) -> Vec<Lost<'tests, 'file>> {
        let pooled: Vec<Lost<'tests, 'file>> = before
            .iter()
            .flat_map(|test| test.checks().iter().map(|check| Lost { test, check: check.clone() }))
            .collect();
        Matching::unplaced(pooled, after)
    }
}

impl AsRef<Check> for Lost<'_, '_> {
    fn as_ref(&self) -> &Check {
        &self.check
    }
}

impl<'file> From<Lost<'_, 'file>> for Violation<'file> {
    fn from(lost: Lost<'_, 'file>) -> Violation<'file> {
        Violation::new(*lost.test.as_ref(), Weakened(&lost))
    }
}

struct Weakened<'lost, 'tests, 'file>(&'lost Lost<'tests, 'file>);

impl fmt::Display for Weakened<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "test {} lost a check: {} has no check at least as strong after the change",
            self.0.test.identity(),
            self.0.check
        )
    }
}

impl Finding for Weakened<'_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "restore the check, or keep one at least as strong (the same or a stronger relation, as many pinned values, as many cases); a deliberate narrowing passes only when the PR cites an owner-approved spec detail that names the test as <package>::<test>",
        )
    }
}
