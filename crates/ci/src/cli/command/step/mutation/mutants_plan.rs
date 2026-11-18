use crate::cli::command::Telling;
use console_ports::Shown;
mod boundary;
mod incremental;
mod tested;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::super::journal::Journal;
use super::Step;
use super::kept::Mutant;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Flag};
use crate::cli::ports::{Exit, Invocation, Ports, Text};
use boundary::Shards;
use incremental::{Choice, Prior};
use tested::{Reach, Reaching, Tested};

#[derive(Debug)]
struct Planned {
    shards: Shards,
    choice: Option<Choice>,
}

#[derive(Debug)]
pub struct MutantsPlan {
    diff: Journal,
    reach: Reach,
    output: Journal,
    prior: Option<Prior>,
    scope: Vec<OsString>,
}

impl MutantsPlan {
    const LISTED: [&'static str; 4] = ["mutants", "--list", "--json", "--in-diff"];
    const PRIOR: Flag = Flag::new("--prior");

    fn planned(&self, ports: &impl Ports) -> Result<Planned, Failure<'_>> {
        let changed =
            Tested::asked(ports, &self.reach, OsStr::new(Tested::PULL_BASE), OsStr::new(Tested::HEAD))?.diff(ports)?;
        self.diff.replace(ports.files(), &changed)?;
        let diff: &Path = self.diff.as_ref();
        let choice = self.prior.as_ref().map(|prior| prior.chosen(ports, &self.reach, diff)).transpose()?;
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
}

impl Step for MutantsPlan {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsPlan, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let diff = &arguments.taken()?;
        let output = &arguments.taken()?;
        let reaching = arguments
            .taken()?
            .to_str()
            .ok_or(Failure::Usage)?
            .parse::<Reaching>()
            .map_err(|_unreached| Failure::Usage)?;
        let reach = Reach::taken(reaching, &mut arguments)?;
        let flagged = arguments.flagged(Self::PRIOR)?;
        let directory = flagged.path();
        let mut rest = flagged.following();
        let metadata = directory.as_ref().map(|_| rest.taken()).transpose()?;
        let prior = directory.zip(metadata).map(|(found, read)| Prior::new(found, PathBuf::from(read)));
        Ok(MutantsPlan {
            diff: Journal::from(PathBuf::from(diff)),
            reach,
            output: Journal::file(Path::new(output)),
            prior,
            scope: rest.rest(),
        })
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
