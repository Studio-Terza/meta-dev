use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult {
    name: String,
    summary: String,
    failed: Vec<String>,
}

impl MutantResult {
    pub fn new(name: String, summary: String, failed: Vec<String>) -> MutantResult {
        MutantResult { name, summary, failed }
    }
}
