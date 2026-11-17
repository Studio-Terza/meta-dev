use newtype::{Arbitrary, Newtype};
use std::fmt;

use serde::Serialize;

#[derive(Newtype, Arbitrary, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TestName(String);

impl fmt::Display for TestName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
