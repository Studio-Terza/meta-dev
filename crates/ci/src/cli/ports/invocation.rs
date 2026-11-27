use std::ffi::{OsStr, OsString};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    program: OsString,
    arguments: Vec<OsString>,
}

impl Invocation {
    pub fn new(program: impl Into<OsString>, arguments: impl IntoIterator<Item = impl Into<OsString>>) -> Invocation {
        Invocation { program: program.into(), arguments: arguments.into_iter().map(Into::into).collect() }
    }

    #[must_use]
    pub fn program(&self) -> &OsStr {
        &self.program
    }

    #[must_use]
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }
}
