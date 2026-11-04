use std::fmt;

use serde::Serialize;

use super::test_name::TestName;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct FailedTest<'log>(&'log TestName);

impl<'log> From<&'log TestName> for FailedTest<'log> {
    fn from(name: &'log TestName) -> FailedTest<'log> {
        FailedTest(name)
    }
}

impl fmt::Display for FailedTest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
