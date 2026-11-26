use crate::cli::command::{Telling, Told};
mod keeping;
mod outcome;
mod pass;
mod reruns;
mod selected;
mod verdict;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::super::journal::Journal;
use super::Step;
use super::boundary::Shard;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Flag, Journaled, Location, Parsing};
use crate::cli::ports::{Exit, Ports, Text};
use grammar_output::Record;
use keeping::Keeping;
use pass::Pass;
pub(crate) use reruns::Deferred;
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
            Ok(Telling::deferring(vec![Told::Said(timed_out)], Deferred::new(reruns, survivors)))
        })
    }

    fn flagged(head: ChangedHead, given: Arguments) -> Result<MutantsChanged, Failure<'static>> {
        given.flagged::<Journaled>(Self::SELECTED)?.concluded(|selected, following| {
            Ok(following.flagged::<Location>(Self::KEPT)?.concluded(|kept, rest| MutantsChanged {
                diff: head.diff,
                narrowed: Journal::from(head.output.join(Self::NARROWED)),
                chosen: Journal::from(head.output.join(Self::CHOSEN)),
                scoped: Pass::within(head.output.join(Self::SCOPED)),
                own: Pass::within(head.output),
                shard: head.shard,
                selected,
                keeping: Keeping::from(kept),
                scope: rest.rest(),
            }))
        })
    }
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct ChangedHead {
    #[field(Journaled)]
    diff: Journal,
    #[field(Location)]
    output: PathBuf,
    #[field(Parsing<Shard>)]
    shard: Shard,
}

impl Step for MutantsChanged {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsChanged, Failure<'static>> {
        ChangedHead::read(Arguments::from(given))?.concluded(MutantsChanged::flagged)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        match self.scope.first() {
            None => self.keeping.concluded(ports, Verdict::Skipped),
            Some(_) => self.tested(ports),
        }
    }
}
