use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct Logged {
    log: PathBuf,
    invocation: Invocation,
}

impl Step for Logged {
    fn from_arguments(arguments: &[OsString]) -> Result<Logged, Failure> {
        match arguments {
            [log, program, rest @ ..] => {
                Ok(Logged { log: PathBuf::from(log), invocation: Invocation::new(program, rest) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        ports
            .logged(&self.invocation, &self.log)
            .map(|run| run.exit())
            .map_err(|source| Failure::Program { program: self.invocation.program().to_owned(), source })
    }
}
