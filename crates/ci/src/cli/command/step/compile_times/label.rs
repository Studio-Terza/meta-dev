use std::fmt;

use newtype::Newtype;
use serde::Deserialize;

#[derive(Newtype, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(String);

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
