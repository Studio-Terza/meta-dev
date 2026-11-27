use std::path::{Path, PathBuf};

use newtype::{Display, Newtype};
use serde::Deserialize;

use crate::Deadline;

#[derive(Newtype, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Printed(String);

#[derive(Newtype, Display, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Spoken(String);

impl From<&Call> for Spoken {
    fn from(call: &Call) -> Spoken {
        Spoken([call.program].into_iter().chain(call.arguments.iter().map(String::as_str)).collect::<Vec<&str>>().join(" "))
    }
}

impl AsRef<str> for Printed {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    program: &'static str,
    arguments: Vec<String>,
    directory: PathBuf,
    deadline: Deadline,
}

impl Call {
    #[must_use]
    pub fn new(program: &'static str, arguments: Vec<String>, directory: PathBuf, deadline: Deadline) -> Call {
        Call { program, arguments, directory, deadline }
    }

    #[must_use]
    pub fn program(&self) -> &'static str {
        self.program
    }

    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn deadline(&self) -> Deadline {
        self.deadline
    }
}
