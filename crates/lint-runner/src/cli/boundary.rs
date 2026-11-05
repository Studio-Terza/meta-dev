use std::fmt;

use lint_core::Report;
use lint_layout::Failure;

#[derive(Debug)]
pub(crate) struct Rendered {
    slot: usize,
    text: String,
}

impl Rendered {
    pub(crate) fn render(slot: usize, report: &Report<'_>) -> Rendered {
        Rendered { slot, text: report.to_string() }
    }

    pub(crate) fn failed(slot: usize, failure: &Failure) -> Rendered {
        Rendered { slot, text: format!("{failure}\n") }
    }

    pub(crate) fn slot(&self) -> usize {
        self.slot
    }
}

impl fmt::Display for Rendered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}
