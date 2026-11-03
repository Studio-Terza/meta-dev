use serde::Serialize;
use test_log::TestName;

use super::outcomes::Title;
use super::verdict::Verdict;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MutantResult<'log> {
    name: Title,
    summary: Verdict,
    failed: Vec<&'log TestName>,
}

impl<'log> MutantResult<'log> {
    pub(crate) fn new(name: Title, summary: Verdict, failed: Vec<&'log TestName>) -> MutantResult<'log> {
        MutantResult { name, summary, failed }
    }
}
