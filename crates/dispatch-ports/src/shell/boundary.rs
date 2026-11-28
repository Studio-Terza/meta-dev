use std::path::{Path, PathBuf};

use newtype::{Display, Newtype};

use crate::Deadline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    program: &'static str,
    arguments: Vec<String>,
    directory: PathBuf,
    deadline: Deadline,
    fed: Option<String>,
}

#[derive(Newtype, Display, Clone, Debug, Default, PartialEq, Eq)]
pub struct Printed(String);

impl Call {
    #[must_use]
    pub fn new(program: &'static str, arguments: Vec<String>, directory: PathBuf, deadline: Deadline) -> Call {
        Call { program, arguments, directory, deadline, fed: None }
    }

    #[must_use]
    pub fn fed(self, input: String) -> Call {
        Call { fed: Some(input), ..self }
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

    #[must_use]
    pub fn input(&self) -> Option<&str> {
        self.fed.as_deref()
    }
}

impl AsRef<str> for Printed {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
