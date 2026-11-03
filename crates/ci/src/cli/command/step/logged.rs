use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct Logged {
    log: PathBuf,
    invocation: Invocation,
}

impl Step for Logged {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Logged, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let log = &arguments.taken()?;
        let program = &arguments.taken()?;
        let rest = arguments.rest();
        Ok(Logged { log: PathBuf::from(log), invocation: Invocation::new(program, rest) })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        ports
            .logged(&self.invocation, &self.log)
            .map(|run| run.exit())
            .map_err(|source| Failure::Program { program: self.invocation.program().to_owned(), source })
    }
}
