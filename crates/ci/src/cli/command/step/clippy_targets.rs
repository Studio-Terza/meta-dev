mod scope;
mod workspace;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::Changed;
use super::measured::Measured;
use super::stage::Stage;
use super::timed::Timed;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use scope::Scope;
use workspace::Workspace;

#[derive(Debug)]
pub struct ClippyTargets {
    measured: Measured,
    stage: Stage,
    arguments: Vec<OsString>,
}

impl ClippyTargets {
    const CLIPPY: [&'static str; 2] = ["clippy", "--locked"];
}

impl Step for ClippyTargets {
    fn from_arguments(words: Vec<OsString>) -> Result<ClippyTargets, Failure<'static>> {
        let mut arguments = Arguments::from(words);
        let measured = &arguments.taken()?;
        let stage = &arguments.taken()?;
        Ok(ClippyTargets {
            measured: Measured::file(Path::new(measured)),
            stage: Stage::new(stage.to_str().ok_or(Failure::Usage)?.to_owned()),
            arguments: arguments.rest(),
        })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let workspace = Changed::workspace_metadata(ports)?.as_ref().parse::<Workspace>()?;
        let targets = workspace.targets(&Scope::from(self.arguments.as_slice()));
        let invocation = Invocation::new(
            "cargo",
            Self::CLIPPY
                .iter()
                .map(OsString::from)
                .chain(targets.into_iter().map(OsString::from))
                .chain(self.arguments.iter().cloned()),
        );
        Timed::measure(&self.measured, &self.stage, &invocation, ports)
    }
}
