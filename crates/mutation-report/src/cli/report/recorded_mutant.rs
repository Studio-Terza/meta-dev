mod verdict;

use std::fmt;

use serde::Deserialize;
use text_lines::OneLine;

pub use verdict::Verdict;

#[derive(Clone, Debug, Deserialize)]
pub struct RecordedMutant {
    name: OneLine,
    summary: Verdict,
    failed: Vec<OneLine>,
}

impl RecordedMutant {
    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub(crate) fn has_summary(&self, summary: Verdict) -> bool {
        self.summary == summary
    }

    pub(crate) fn failed(&self) -> &[OneLine] {
        &self.failed
    }
}

impl fmt::Display for RecordedMutant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failed: Vec<String> = self.failed.iter().map(ToString::to_string).collect();
        write!(formatter, "{} -> {}", self.name, failed.join(", "))
    }
}
