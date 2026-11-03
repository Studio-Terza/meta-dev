use lint_core::Report;

use crate::facts::Facts;

pub trait FactsLint {
    fn report_facts<'file>(facts: &Facts<'file>) -> Report<'file>;
}
