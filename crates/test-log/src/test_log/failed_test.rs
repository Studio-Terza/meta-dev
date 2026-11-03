use std::fmt;

use serde::Serialize;

use super::test_name::TestName;

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct FailedTest(TestName);

impl From<&TestName> for FailedTest {
    fn from(name: &TestName) -> FailedTest {
        FailedTest(name.clone())
    }
}

impl fmt::Display for FailedTest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
