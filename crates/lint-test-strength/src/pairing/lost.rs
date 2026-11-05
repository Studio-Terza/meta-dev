use std::fmt;

use lint_core::{Finding, Fix, Violation};
use test_facts::{Claim, Claims, Test};

#[derive(Debug)]
pub(crate) struct Lost<'tests, 'file> {
    test: &'tests Test<'file>,
    claim: &'tests Claim,
}

impl<'tests, 'file> Lost<'tests, 'file> {
    pub(crate) fn in_package(test: &'tests Test<'file>, pool: &Claims) -> Vec<Lost<'tests, 'file>> {
        pool.lost(test.claims()).into_iter().map(|claim| Lost { test, claim }).collect()
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
            "test {} lost a claim: {}; no claim after the change in its package entails it",
            self.0.test, self.0.claim
        )
    }
}

impl Finding for Weakened<'_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "restore the claim, or state one that entails it (the same subject, a value set inside the old one, a domain that covers the old one)",
        )
    }
}
