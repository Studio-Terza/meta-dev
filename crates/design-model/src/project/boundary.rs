use std::str::FromStr;

use grammar::project::Names;
use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype, Spelled};
use serde::Serialize;

#[derive(Newtype, Arbitrary, Display, Spelled, CheckedDeserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[A-Za-z0-9][A-Za-z0-9._-]{0,15}")]
pub struct Project(String);

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a project name, which is letters, digits, '.', '_' and '-', starting with a letter or digit")]
pub struct Unnamed(String);

impl Checked for Project {
    type Error = Unnamed;

    fn checked(text: &String) -> Result<&String, Unnamed> {
        Names::named(text).map(|_named| text).ok_or_else(|| Unnamed(text.to_owned()))
    }
}

impl FromStr for Project {
    type Err = Unnamed;

    fn from_str(text: &str) -> Result<Project, Unnamed> {
        Project::try_new(text.to_owned())
    }
}
