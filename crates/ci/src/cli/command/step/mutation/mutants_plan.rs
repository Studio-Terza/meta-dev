mod boundary;
mod tested;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use text_lines::Lines;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use boundary::Shards;
use tested::Tested;

#[derive(Debug)]
pub struct MutantsPlan {
    diff: Journal,
    lint: PathBuf,
    output: Journal,
    scope: Vec<OsString>,
}

impl MutantsPlan {
    const LISTED: [&'static str; 3] = ["mutants", "--list", "--in-diff"];

    fn planned(&self, ports: &impl Ports) -> Result<Shards, Failure<'_>> {
        let changed = Tested::asked(ports, &self.lint)?.diff(ports)?;
        self.diff.replace(ports.files(), &changed)?;
        let diff: &Path = self.diff.as_ref();
        let invocation =
            Invocation::new("cargo", Self::LISTED.map(OsString::from).into_iter().chain([diff.as_os_str().to_owned()]));
        let listed = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        Ok(Shards::for_mutants(listed.as_ref().parse::<Lines>().into_iter().flatten().count()))
    }
}

impl Step for MutantsPlan {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsPlan, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let diff = &arguments.taken()?;
        let lint = &arguments.taken()?;
        let output = &arguments.taken()?;
        let scope = arguments.rest();
        Ok(MutantsPlan {
            diff: Journal::from(PathBuf::from(diff)),
            lint: PathBuf::from(lint),
            output: Journal::file(Path::new(output)),
            scope,
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let shards = self.scope.first().map_or(Ok(Shards::for_mutants(0)), |_| self.planned(ports))?;
        writeln!(out, "{shards}")?;
        self.output.append(ports.files(), format_args!("shards={}\n", shards.indices()))?;
        Ok(Exit::Success)
    }
}
