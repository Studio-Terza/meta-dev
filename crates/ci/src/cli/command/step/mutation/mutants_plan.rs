use crate::cli::command::Telling;
use console_ports::Shown;
mod boundary;
mod incremental;
mod tested;

use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::super::journal::Journal;
use super::Step;
use super::kept::Mutant;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Flag, Journaled, Location};
use crate::cli::ports::{Exit, Invocation, Ports, Text};
use boundary::Shards;
use grammar_output::Record;
use incremental::{Choice, Prior};
use tested::Tested;

#[derive(Debug)]
struct Planned {
    shards: Shards,
    choice: Option<Choice>,
}

#[derive(Debug)]
pub struct MutantsPlan {
    diff: Journal,
    output: Journal,
    prior: Option<Prior>,
    scope: Vec<OsString>,
}

impl MutantsPlan {
    const LISTED: [&'static str; 4] = ["mutants", "--list", "--json", "--in-diff"];
    const PRIOR: Flag = Flag::new("--prior");

    fn planned(&self, ports: &impl Ports) -> Result<Planned, Failure<'_>> {
        let changed = Tested::asked(ports, OsStr::new(Tested::PULL_BASE), OsStr::new(Tested::HEAD))?.diff(ports)?;
        self.diff.replace(ports.files(), &changed)?;
        let diff: &Path = self.diff.as_ref();
        let choice = self.prior.as_ref().map(|prior| prior.chosen(ports, diff)).transpose()?;
        let shards = choice.as_ref().and_then(Choice::shards).map_or_else(|| self.listed(ports), Ok)?;
        Ok(Planned { shards, choice })
    }

    fn listed(&self, ports: &impl Ports) -> Result<Shards, Failure<'_>> {
        let diff: &Path = self.diff.as_ref();
        let invocation =
            Invocation::new("cargo", Self::LISTED.map(OsString::from).into_iter().chain([diff.as_os_str().to_owned()]));
        let listed = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let costs = self.prior.as_ref().map(|prior| prior.costs(ports)).transpose()?.unwrap_or_default();
        let mutants = Mutant::listed(&Text::new(listed.as_ref().to_owned())).map_err(Failure::Outcomes)?;
        Ok(Shards::for_mutants(mutants.iter().map(|mutant| costs.per_mutant(mutant)).collect()))
    }

    fn headed(head: PlanHead, given: Arguments) -> Result<MutantsPlan, Failure<'static>> {
        let directory = given.flagged::<Location>(Self::PRIOR)?;
        let read = directory.concluded(Prior::following)?;
        Ok(read.concluded(|prior, rest| MutantsPlan {
            diff: head.diff,
            output: head.output,
            prior,
            scope: rest.rest(),
        }))
    }
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct PlanHead {
    #[field(Journaled)]
    diff: Journal,
    #[field(Journaled)]
    output: Journal,
}

impl Step for MutantsPlan {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsPlan, Failure<'static>> {
        PlanHead::read(Arguments::from(given))?.concluded(MutantsPlan::headed)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let Planned { shards, choice } = self
            .scope
            .first()
            .map_or(Ok(Planned { shards: Shards::for_mutants(Vec::new()), choice: None }), |_| self.planned(ports))?;
        let said = choice.iter().map(Shown::said).chain([Shown::said(format_args!("{shards}\n"))]).collect();
        Ok(Telling::reported(
            said,
            self.output.append(ports.files(), format_args!("shards={}\n", shards.matrix())).map(|()| Exit::Success),
        ))
    }
}
