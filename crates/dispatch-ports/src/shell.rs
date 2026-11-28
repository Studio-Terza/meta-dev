use std::path::{Path, PathBuf};

use newtype::{Display, Newtype};
use port_core::port;

use timer_ports::Deadline;

use crate::DispatchFailure;

#[port]
pub trait Shell {
    fn ran(&self, call: &Call) -> Result<Ran, DispatchFailure>;

    fn fanned(&self, calls: &[Call]) -> Vec<Result<Ran, DispatchFailure>>;
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ran {
    Succeeded(Printed),
    Failed(Printed),
    Overdue,
}

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

impl Ran {
    #[must_use]
    pub fn printed(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) | Ran::Failed(printed) => Some(printed),
            Ran::Overdue => None,
        }
    }

    #[must_use]
    pub fn succeeded(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) => Some(printed),
            Ran::Failed(_) | Ran::Overdue => None,
        }
    }
}

impl AsRef<str> for Printed {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
