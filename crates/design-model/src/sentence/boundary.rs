use std::str::FromStr;

use grammar_line::Unbroken;
use newtype::{Display, Spelled};

#[derive(Display, Spelled, Clone, Debug, PartialEq, Eq)]
pub struct Condition(String);

#[derive(Display, Spelled, Clone, Debug, PartialEq, Eq)]
pub struct Reason(String);

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not one line of text: it is empty, blank or broken across lines")]
pub struct Unstated(String);

impl FromStr for Condition {
    type Err = Unstated;

    fn from_str(text: &str) -> Result<Condition, Unstated> {
        Unbroken::spelled(text).map(str::to_owned).map(Condition).ok_or_else(|| Unstated(text.to_owned()))
    }
}

impl FromStr for Reason {
    type Err = Unstated;

    fn from_str(text: &str) -> Result<Reason, Unstated> {
        Unbroken::spelled(text).map(str::to_owned).map(Reason).ok_or_else(|| Unstated(text.to_owned()))
    }
}
