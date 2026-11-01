use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct RecordedMutant {
    name: String,
    summary: String,
    failed: Vec<String>,
}

impl RecordedMutant {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn has_summary(&self, summary: &str) -> bool {
        self.summary == summary
    }

    pub(crate) fn failed(&self) -> &[String] {
        &self.failed
    }
}

impl fmt::Display for RecordedMutant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.name, self.failed.join(", "))
    }
}
