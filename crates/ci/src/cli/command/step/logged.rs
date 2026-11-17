use crate::cli::command::Telling;
use std::ffi::OsString;
use std::path::PathBuf;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Invocation, Ports};

#[derive(Debug)]
pub struct Logged {
    log: PathBuf,
    invocation: Invocation,
}

impl Step for Logged {
    fn from_arguments(given: Vec<OsString>) -> Result<Logged, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let log = &arguments.taken()?;
        let program = &arguments.taken()?;
        let rest = arguments.rest();
        Ok(Logged { log: PathBuf::from(log), invocation: Invocation::new(program, rest) })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        Ok(Telling::ended(
            ports
                .logged(&self.invocation, &self.log)
                .map(|run| run.exit())
                .map_err(|source| Failure::Program { program: self.invocation.program().to_owned(), source }),
        ))
    }
}
