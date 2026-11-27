use crate::cli::command::Telling;
mod name;
mod scope;
mod workspace;

use std::ffi::OsString;

use super::Step;
use super::changed::Changed;
use super::measured::Measured;
use super::stage::Stage;
use super::timed::Timed;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Recording, Utf8, Word};
use crate::cli::ports::{Invocation, Ports};
use grammar_output::Record;
use scope::Scope;
use workspace::Workspace;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct ClippyTargets {
    #[field(Recording)]
    measured: Measured,
    #[field(Utf8, via = Stage::new)]
    stage: Stage,
    #[repeated(Word)]
    arguments: Vec<OsString>,
}

impl ClippyTargets {
    const CLIPPY: [&'static str; 2] = ["clippy", "--locked"];
}

impl Step for ClippyTargets {
    fn from_arguments(words: Vec<OsString>) -> Result<ClippyTargets, Failure<'static>> {
        ClippyTargets::read(Arguments::from(words))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let workspace = Changed::workspace_metadata(ports)?.as_ref().parse::<Workspace>()?;
        let targets = workspace.targets(&Scope::try_from(self.arguments.as_slice())?)?;
        let invocation = Invocation::new(
            "cargo",
            Self::CLIPPY.iter().map(OsString::from).chain(targets).chain(self.arguments.iter().cloned()),
        );
        Ok(Telling::ended(Timed::measure(&self.measured, &self.stage, &invocation, ports)))
    }
}
