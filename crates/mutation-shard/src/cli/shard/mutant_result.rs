use std::fmt;

use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult {
    name: String,
    summary: String,
    failed: Vec<String>,
}

impl MutantResult {
    pub fn named(name: &impl fmt::Display, summary: String, failed: Vec<String>) -> MutantResult {
        MutantResult { name: name.to_string(), summary, failed }
    }
}

impl fmt::Display for MutantResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}
