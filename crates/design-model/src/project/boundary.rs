use std::str::FromStr;

use grammar_project::Names;
use newtype::{Display, Spelled};
use serde::{Deserialize, Serialize};

#[derive(Display, Spelled, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Project(String);

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a project name, which is letters, digits, '.', '_' and '-', starting with a letter or digit")]
pub struct Unnamed(String);

impl TryFrom<String> for Project {
    type Error = Unnamed;

    fn try_from(text: String) -> Result<Project, Unnamed> {
        Names::named(&text).map(str::to_owned).map(Project).ok_or(Unnamed(text))
    }
}

impl FromStr for Project {
    type Err = Unnamed;

    fn from_str(text: &str) -> Result<Project, Unnamed> {
        Project::try_from(text.to_owned())
    }
}
