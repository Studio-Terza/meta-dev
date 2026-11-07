use serde::Serialize;
use test_log::FailedTest;

use super::title::Title;
use super::verdict::Verdict;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult<'log> {
    name: Title,
    summary: Verdict,
    failed: Vec<FailedTest<'log>>,
}

impl<'log> MutantResult<'log> {
    pub(crate) const fn new(name: Title, summary: Verdict, failed: Vec<FailedTest<'log>>) -> Self {
        MutantResult { name, summary, failed }
    }
}
