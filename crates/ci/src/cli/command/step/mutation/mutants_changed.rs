use crate::cli::command::{Telling, Told};
mod pass;
mod verdict;

use std::ffi::{OsStr, OsString};
use std::iter;
use std::path::{Path, PathBuf};

use super::super::journal::Journal;
use super::Step;
use super::boundary::Shard;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};
use pass::{Pass, Survivors};
use verdict::Verdict;

#[derive(Debug)]
pub struct MutantsChanged {
    diff: Journal,
    own: Pass,
    scoped: Pass,
    shard: Shard,
    scope: Vec<OsString>,
}

impl MutantsChanged {
    const IN_DIFF: &'static str = "--in-diff";
    const SCOPED: &'static str = "scoped";

    fn new(diff: PathBuf, output: PathBuf, shard: Shard, scope: Vec<OsString>) -> MutantsChanged {
        MutantsChanged {
            diff: Journal::from(diff),
            scoped: Pass::within(output.join(Self::SCOPED)),
            own: Pass::within(output),
            shard,
            scope,
        }
    }

    fn tested<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let diff: &Path = self.diff.as_ref();
        let shard = self.shard.arguments();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(shard.iter().map(OsString::as_os_str))
            .collect();
        let own = self.own.ran(ports, &selection, &[CargoMutants::MISSED, CargoMutants::TIMEOUT])?;
        Some(own).filter(|exit| *exit == Exit::Success).map_or(Ok(Telling::ended(Ok(own))), |_| {
            let timed_out = self.own.timed_out(ports.files())?;
            let survivors = self.own.survivors(ports.files())?;
            let rest = iter::once_with(move || self.after(ports, survivors.as_ref())).flatten();
            Ok(Telling::lazy(iter::once(Told::Said(timed_out)).chain(rest)))
        })
    }

    fn after<'step>(&'step self, ports: &'step impl Ports, survivors: Option<&Survivors>) -> Vec<Told<'step>> {
        survivors.map_or_else(
            || Verdict::NoneMissed.told(),
            |found| self.rerun(ports, found).unwrap_or_else(|failure| vec![Told::Ended(Err(failure))]),
        )
    }

    fn rerun<'step>(&'step self, ports: &impl Ports, missed: &Survivors) -> Result<Vec<Told<'step>>, Failure<'step>> {
        let diff: &Path = self.diff.as_ref();
        let named = missed.examined();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(named.iter().map(OsString::as_os_str))
            .chain(self.scope.iter().map(OsString::as_os_str))
            .collect();
        let scoped = self.scoped.ran(ports, &selection, &[CargoMutants::TIMEOUT])?;
        Some(scoped)
            .filter(|exit| *exit == Exit::Success)
            .map_or(Ok(vec![Told::Ended(Ok(scoped))]), |_| self.scoped.judged(ports.files(), missed))
    }
}

impl Step for MutantsChanged {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsChanged, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let diff = &arguments.taken()?;
        let output = &arguments.taken()?;
        let shard = arguments.taken()?.to_str().ok_or(Failure::Usage)?.parse()?;
        let scope = arguments.rest();
        Ok(MutantsChanged::new(PathBuf::from(diff), PathBuf::from(output), shard, scope))
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        match self.scope.first() {
            None => Ok(Telling::lazy(Verdict::Skipped.told().into_iter())),
            Some(_) => self.tested(ports),
        }
    }
}
