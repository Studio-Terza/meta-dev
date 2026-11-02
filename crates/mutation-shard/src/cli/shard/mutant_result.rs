use serde::Serialize;
use test_log::TestName;

use super::verdict::Verdict;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult {
    name: String,
    summary: Verdict,
    failed: Vec<TestName>,
}

impl MutantResult {
    pub(crate) fn new(name: String, summary: Verdict, failed: Vec<TestName>) -> MutantResult {
        MutantResult { name, summary, failed }
    }
}
