use crate::cli::command::Telling;
use std::ffi::OsString;

use super::Step;
use super::measured::Measured;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Recording, Utf8, Word};
use crate::cli::ports::{Exit, Invocation, Ports};
use grammar_output::Record;

#[derive(Debug)]
pub(crate) struct Timed {
    measured: Measured,
    stage: Stage,
    invocation: Invocation,
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct TimedWords {
    #[field(Recording)]
    measured: Measured,
    #[field(Utf8, via = Stage::new)]
    stage: Stage,
    #[field(Word)]
    program: OsString,
    #[repeated(Word)]
    rest: Vec<OsString>,
}

impl From<TimedWords> for Timed {
    fn from(words: TimedWords) -> Timed {
        Timed { measured: words.measured, stage: words.stage, invocation: Invocation::new(words.program, words.rest) }
    }
}

impl Step for Timed {
    fn from_arguments(given: Vec<OsString>) -> Result<Timed, Failure<'static>> {
        TimedWords::read(Arguments::from(given))?.ended().map(Timed::from)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        Ok(Telling::ended(Timed::measure(&self.measured, &self.stage, &self.invocation, ports)))
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
