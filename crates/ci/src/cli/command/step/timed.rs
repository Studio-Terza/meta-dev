use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::measured::Measured;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct Timed {
    measured: Measured,
    stage: Stage,
    invocation: Invocation,
}

impl Step for Timed {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let measured = &arguments.taken()?;
        let stage = &arguments.taken()?;
        let program = &arguments.taken()?;
        let rest = arguments.rest();
        Ok(Self {
            measured: Measured::file(Path::new(measured)),
            stage: Stage::new(stage.to_str().ok_or(Failure::Usage)?.to_owned()),
            invocation: Invocation::new(program, rest),
        })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let run = ports
            .run(&self.invocation)
            .map_err(|source| Failure::Program { program: self.invocation.program().to_owned(), source })?;
        self.measured.append(ports.files(), &self.stage, Seconds::from(run.elapsed()))?;
        Ok(run.exit())
    }
}
