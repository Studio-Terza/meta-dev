use crate::cli::command::{Telling, Told};
mod keeping;
mod pass;
mod reruns;
mod selected;
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
use crate::cli::command::step::Flag;
use crate::cli::ports::{Exit, Ports, Text};
use keeping::Keeping;
use pass::Pass;
use reruns::Reruns;
use selected::Selected;
use verdict::Verdict;

#[derive(Debug)]
pub struct MutantsChanged {
    diff: Journal,
    narrowed: Journal,
    chosen: Journal,
    own: Pass,
    scoped: Pass,
    shard: Shard,
    selected: Option<Journal>,
    keeping: Keeping,
    scope: Vec<OsString>,
}

impl MutantsChanged {
    const IN_DIFF: &'static str = "--in-diff";
    const SCOPED: &'static str = "scoped";
    const NARROWED: &'static str = "survivors.diff";
    const CHOSEN: &'static str = "selected.diff";
    const SELECTED: Flag = Flag::new("--selected");
    const KEPT: Flag = Flag::new("--kept");

    fn tested<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let selected = Selected::read(ports, self.selected.as_ref())?;
        let names = selected.named();
        selected.examined().map_or_else(
            || self.keeping.concluded(ports, Verdict::Unchanged),
            |examined| self.mutated(ports, &examined, &names),
        )
    }

    fn mutated<'step>(
        &'step self,
        ports: &'step impl Ports,
        examined: &[OsString],
        names: &[Text],
    ) -> Result<Telling<'step>, Failure<'step>> {
        let chosen = names
            .first()
            .map(|_| Pass::chosen(ports, self.diff.as_ref(), names))
            .transpose()?
            .map(|text| self.chosen.replace(ports.files(), &text))
            .transpose()?;
        let diff: &Path = chosen.map_or_else(|| self.diff.as_ref(), |()| self.chosen.as_ref());
        let shard = self.shard.arguments();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(examined.iter().map(OsString::as_os_str))
            .chain(shard.iter().map(OsString::as_os_str))
            .collect();
        let own = self.own.ran(ports, &selection, &[CargoMutants::MISSED, CargoMutants::TIMEOUT])?;
        Some(own).filter(|exit| *exit == Exit::Success).map_or(Ok(Telling::ended(Ok(own))), |_| {
            let timed_out = self.own.timed_out(ports.files())?;
            let survivors = self.own.survivors(ports.files())?;
            survivors
                .as_ref()
                .map(|found| self.own.narrowed(ports.files(), found))
                .transpose()?
                .map_or(Ok(()), |narrowed| self.narrowed.replace(ports.files(), &narrowed))?;
            let reruns = Reruns::new(&self.narrowed, &self.scoped, &self.scope, &self.keeping);
            let rest = iter::once_with(move || reruns.after(ports, survivors.as_ref())).flatten();
            Ok(Telling::lazy(iter::once(Told::Said(timed_out)).chain(rest)))
        })
    }
}

impl Step for MutantsChanged {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsChanged, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let diff = PathBuf::from(arguments.taken()?);
        let output = PathBuf::from(arguments.taken()?);
        let shard = arguments.taken()?.to_str().ok_or(Failure::Usage)?.parse()?;
        let selected = arguments.flagged(Self::SELECTED)?;
        let kept = selected.path().map(Journal::from);
        let keeping = selected.following().flagged(Self::KEPT)?;
        Ok(MutantsChanged {
            diff: Journal::from(diff),
            narrowed: Journal::from(output.join(Self::NARROWED)),
            chosen: Journal::from(output.join(Self::CHOSEN)),
            scoped: Pass::within(output.join(Self::SCOPED)),
            own: Pass::within(output),
            shard,
            selected: kept,
            keeping: Keeping::from(keeping.path()),
            scope: keeping.following().rest(),
        })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        match self.scope.first() {
            None => self.keeping.concluded(ports, Verdict::Skipped),
            Some(_) => self.tested(ports),
        }
    }
}
