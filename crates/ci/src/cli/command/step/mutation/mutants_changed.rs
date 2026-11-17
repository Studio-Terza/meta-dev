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
use crate::cli::ports::{Exit, Ports};
use keeping::Keeping;
use pass::Pass;
use reruns::Reruns;
use selected::Selected;
use verdict::Verdict;

#[derive(Debug)]
pub struct MutantsChanged {
    diff: Journal,
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
    const SELECTED: Flag = Flag::new("--selected");
    const KEPT: Flag = Flag::new("--kept");

    fn tested<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let selected = Selected::read(ports, self.selected.as_ref())?;
        selected.examined().map_or_else(
            || self.keeping.concluded(ports, Verdict::Unchanged),
            |examined| self.mutated(ports, &examined),
        )
    }

    fn mutated<'step>(
        &'step self,
        ports: &'step impl Ports,
        examined: &[OsString],
    ) -> Result<Telling<'step>, Failure<'step>> {
        let diff: &Path = self.diff.as_ref();
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
            let reruns = Reruns::new(&self.diff, &self.scoped, &self.scope, &self.keeping);
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
        let selected = arguments.flagged(Self::SELECTED)?.map(|path| Journal::from(PathBuf::from(path)));
        let kept = arguments.flagged(Self::KEPT)?.map(PathBuf::from);
        Ok(MutantsChanged {
            diff: Journal::from(diff),
            scoped: Pass::within(output.join(Self::SCOPED)),
            own: Pass::within(output),
            shard,
            selected,
            keeping: Keeping::from(kept),
            scope: arguments.rest(),
        })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        match self.scope.first() {
            None => self.keeping.concluded(ports, Verdict::Skipped),
            Some(_) => self.tested(ports),
        }
    }
}
