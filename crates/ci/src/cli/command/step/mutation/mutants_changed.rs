mod pass;
mod verdict;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::super::journal::Journal;
use super::Step;
use super::boundary::{Exact, Shard};
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
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
    const DIFF: [&'static str; 6] = ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"];
    const SCOPED: &'static str = "scoped";
    const EXAMINE: &'static str = "--re";

    fn new(diff: PathBuf, output: PathBuf, shard: Shard, scope: Vec<OsString>) -> MutantsChanged {
        MutantsChanged {
            diff: Journal::from(diff),
            scoped: Pass::within(output.join(Self::SCOPED)),
            own: Pass::within(output),
            shard,
            scope,
        }
    }

    fn tested(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let invocation = Invocation::new("git", Self::DIFF);
        let changed = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        self.diff.replace(ports.files(), &changed)?;
        let diff: &Path = self.diff.as_ref();
        let shard = self.shard.arguments();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(shard.iter().map(OsString::as_os_str))
            .collect();
        let own = self.own.ran(ports, &selection, &[CargoMutants::MISSED, CargoMutants::TIMEOUT])?;
        own.followed_by(|| {
            let survivors = self.own.survivors(ports.files(), out)?;
            let rerun = survivors.map(|found| self.rerun(ports, out, &found));
            rerun.unwrap_or_else(|| Verdict::NoneMissed.reported(out))
        })
    }

    fn rerun(&self, ports: &impl Ports, out: &mut impl Write, missed: &Survivors) -> Result<Exit, Failure<'_>> {
        let diff: &Path = self.diff.as_ref();
        let named: Vec<OsString> = missed
            .names()
            .iter()
            .flat_map(|name| [OsString::from(Self::EXAMINE), OsString::from(Exact::from(name))])
            .collect();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(named.iter().map(OsString::as_os_str))
            .chain(self.scope.iter().map(OsString::as_os_str))
            .collect();
        let scoped = self.scoped.ran(ports, &selection, &[CargoMutants::TIMEOUT])?;
        scoped.followed_by(|| self.scoped.judged(ports.files(), out, missed))
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

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        match self.scope.first() {
            None => Verdict::Skipped.reported(out),
            Some(_) => self.tested(ports, out),
        }
    }
}
