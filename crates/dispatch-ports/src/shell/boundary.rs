use std::path::{Path, PathBuf};

use newtype::{Display, Newtype};

use crate::Deadline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    program: String,
    arguments: Vec<String>,
    directory: PathBuf,
    deadline: Deadline,
}

#[derive(Newtype, Display, Clone, Debug, Default, PartialEq, Eq)]
pub struct Printed(String);

impl Call {
    #[must_use]
    pub fn new(program: &str, arguments: Vec<String>, directory: PathBuf, deadline: Deadline) -> Call {
        Call { program: program.to_owned(), arguments, directory, deadline }
    }

    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
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

impl AsRef<str> for Printed {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
