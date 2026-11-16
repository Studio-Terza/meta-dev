use graph::Model;

use lint_core::Report;

pub trait ModelLint {
    fn report(model: &Model) -> Report<'_>;
}
