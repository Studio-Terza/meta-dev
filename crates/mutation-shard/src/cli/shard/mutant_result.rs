use serde::Serialize;
use test_log::TestName;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult {
    name: String,
    summary: String,
    failed: Vec<TestName>,
}

impl MutantResult {
    pub(crate) fn new(name: String, summary: String, failed: Vec<TestName>) -> MutantResult {
        MutantResult { name, summary, failed }
    }
}
