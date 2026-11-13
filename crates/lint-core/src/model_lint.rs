use graph::Model;

use crate::report::Report;

pub trait ModelLint {
    fn report(model: &Model) -> Report<'_>;
}
