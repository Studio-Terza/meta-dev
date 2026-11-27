use std::path::{Path, PathBuf};

use crate::Deadline;

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
