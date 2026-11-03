use lint_core::Report;

use crate::facts::Facts;

pub trait FactsLint {
    fn report_facts(facts: &Facts) -> Report;
}
