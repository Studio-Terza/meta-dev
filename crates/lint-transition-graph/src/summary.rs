use lint_core::{Site, Violation};
use lint_layout::Finding;

#[derive(Debug)]
pub(crate) struct Summary;

impl Summary {
    pub(crate) fn placed(first: Option<Site<'_>>, finding: impl Finding) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, finding))
    }
}
