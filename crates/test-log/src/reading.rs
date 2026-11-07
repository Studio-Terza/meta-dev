use std::collections::BTreeMap;

use super::boundary::Binary;
use super::boundary::TestResult;
use super::test_name::TestName;
use super::verdict::Verdict;

#[derive(Debug, Default)]
pub struct Reading {
    binary: Option<Binary>,
    results: BTreeMap<TestName, Verdict>,
}

impl Reading {
    #[must_use]
    pub fn in_binary(self, binary: Binary) -> Self {
        Self { binary: Some(binary), ..self }
    }

    #[must_use]
    pub fn recording(mut self, result: &TestResult) -> Self {
        self.results.insert(result.key(self.binary.as_ref()), result.verdict());
        self
    }

    pub(crate) fn into_results(self) -> BTreeMap<TestName, Verdict> {
        self.results
    }
}
