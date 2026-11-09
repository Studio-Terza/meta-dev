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
    fn from_arguments(given: Vec<OsString>) -> Result<Timed, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let measured = &arguments.taken()?;
        let stage = &arguments.taken()?;
        let program = &arguments.taken()?;
        let rest = arguments.rest();
        Ok(Timed {
            measured: Measured::file(Path::new(measured)),
            stage: Stage::new(stage.to_str().ok_or(Failure::Usage)?.to_owned()),
            invocation: Invocation::new(program, rest),
        })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        Timed::measure(&self.measured, &self.stage, &self.invocation, ports)
    }
}

impl Timed {
    pub(super) fn measure<'measured>(
        measured: &'measured Measured,
        stage: &Stage,
        invocation: &Invocation,
        ports: &impl Ports,
    ) -> Result<Exit, Failure<'measured>> {
        let run = ports
            .run(invocation)
            .map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })?;
        measured.append(ports.files(), stage, Seconds::from(run.elapsed()))?;
        Ok(run.exit())
    }
}
