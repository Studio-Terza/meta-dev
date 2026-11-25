use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::keeping::Keeping;
use super::pass::{Pass, Survivors};
use super::verdict::Verdict;
use crate::cli::command::Told;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::mutation::cargo_mutants::CargoMutants;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub(super) struct Reruns<'step> {
    narrowed: &'step Journal,
    scoped: &'step Pass,
    scope: &'step [OsString],
    keeping: &'step Keeping,
}

impl<'step> Reruns<'step> {
    const IN_DIFF: &'static str = "--in-diff";

    pub(super) fn new(
        narrowed: &'step Journal,
        scoped: &'step Pass,
        scope: &'step [OsString],
        keeping: &'step Keeping,
    ) -> Reruns<'step> {
        Reruns { narrowed, scoped, scope, keeping }
    }

    pub(super) fn after(&self, ports: &impl Ports, survivors: Option<&Survivors>) -> Vec<Told<'step>> {
        survivors
            .map_or_else(
                || self.keeping.kept(ports, None).map(|()| Verdict::NoneMissed.told()),
                |found| self.rerun(ports, found),
            )
            .unwrap_or_else(|failure| vec![Told::Ended(Err(failure))])
    }

    fn rerun(&self, ports: &impl Ports, missed: &Survivors) -> Result<Vec<Told<'step>>, Failure<'step>> {
        let diff: &Path = self.narrowed.as_ref();
        let named = missed.examined();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(named.iter().map(OsString::as_os_str))
            .chain(self.scope.iter().map(OsString::as_os_str))
            .collect();
        let scoped = self.scoped.ran(ports, &selection, &[CargoMutants::TIMEOUT])?;
        [Exit::Success, Exit::Failure(CargoMutants::MISSED)]
            .contains(&scoped)
            .then(|| self.scoped.outcomes())
            .map_or(Ok(()), |outcomes| self.keeping.kept(ports, Some(&outcomes)))?;
        Some(scoped)
            .filter(|exit| *exit == Exit::Success)
            .map_or(Ok(vec![Told::Ended(Ok(scoped))]), |_| self.scoped.judged(ports.files(), missed))
    }
}
