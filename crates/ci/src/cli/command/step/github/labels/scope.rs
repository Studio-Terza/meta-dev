use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::change::Change;
use super::check::Check;
use super::fault::{Fault, Owed};
use super::hub::PullRequest;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::changed::Changed;
use crate::cli::command::step::journal::Journal;
use crate::cli::ports::{Ports, Text};

#[derive(Debug)]
pub struct Scope<'journal> {
    metadata: &'journal Journal,
    sources: Vec<OsString>,
}

impl<'journal> Scope<'journal> {
    const BEYOND: &'static str = "beyond";

    pub(crate) fn reached_by(metadata: &'journal Journal, changes: &[Change]) -> Scope<'journal> {
        let sources = changes
            .iter()
            .filter(|change| Check::ALL.iter().all(|check| check.flagging(change).is_none()))
            .flat_map(Change::paths)
            .map(|path| path.as_os_str().to_owned())
            .collect();
        Scope { metadata, sources }
    }

    pub(crate) fn judged<'change>(
        &self,
        ports: &impl Ports,
        checks: &[Check],
        refactor: Option<&PullRequest>,
        changes: &'change [Change],
    ) -> Result<Vec<Owed<'change>>, Failure<'static>> {
        let asked: Vec<(Check, Vec<&'change Change>)> =
            checks.iter().map(|check| (*check, check.flagged(changes))).collect();
        let produced: BTreeSet<&Path> = asked
            .iter()
            .filter(|(check, _)| !matches!((check, refactor), (Check::Goldens, Some(_))))
            .flat_map(|(_, flagged)| flagged)
            .flat_map(|change| change.paths())
            .collect();
        let questions: Vec<Vec<&OsStr>> = produced.iter().map(|path| self.arguments(path).collect()).collect();
        let answers: BTreeMap<&Path, Text> = produced
            .into_iter()
            .zip(Changed::reaches(ports, &questions))
            .map(|(path, answer)| answer.map(|text| (path, text)))
            .collect::<Result<_, Failure<'static>>>()?;
        Ok(asked.into_iter().map(|(check, flagged)| Self::owed(&answers, check, refactor, flagged)).collect())
    }

    fn owed<'change>(
        answers: &BTreeMap<&Path, Text>,
        check: Check,
        refactor: Option<&PullRequest>,
        flagged: Vec<&'change Change>,
    ) -> Owed<'change> {
        match (check, refactor) {
            (Check::Goldens, Some(_)) => Owed::new(Fault::Refactor, flagged),
            _ => Owed::new(
                Fault::Beyond(check),
                flagged.into_iter().filter(|change| Self::reaching(answers, change).is_none()).collect(),
            ),
        }
    }

    fn reaching<'change>(answers: &BTreeMap<&Path, Text>, change: &'change Change) -> Option<&'change Change> {
        change
            .paths()
            .iter()
            .all(|produced| answers.get(produced).is_some_and(|listed| listed.as_ref().is_empty()))
            .then_some(change)
    }

    fn arguments<'question>(&'question self, produced: &'question Path) -> impl Iterator<Item = &'question OsStr> {
        [OsStr::new(Self::BEYOND), self.metadata.as_ref().as_os_str(), produced.as_os_str()]
            .into_iter()
            .chain(self.sources.iter().map(OsString::as_os_str))
    }
}
