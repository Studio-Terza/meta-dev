use std::fmt;

use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TestName(String);

impl TestName {
    pub(crate) fn new(text: String) -> TestName {
        TestName(text)
    }
}

impl fmt::Display for TestName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
