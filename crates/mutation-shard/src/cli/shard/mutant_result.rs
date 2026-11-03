use serde::Serialize;
use test_log::FailedTest;

use super::outcomes::Title;
use super::verdict::Verdict;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult {
    name: Title,
    summary: Verdict,
    failed: Vec<FailedTest>,
}

impl MutantResult {
    pub(crate) fn new(name: Title, summary: Verdict, failed: Vec<FailedTest>) -> MutantResult {
        MutantResult { name, summary, failed }
    }
}
