use std::ffi::OsString;
use std::path::Path;

use super::change::Change;
use super::check::Check;
use super::fault::{Fault, Owed};
use super::hub::PullRequest;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::changed::Changed;
use crate::cli::command::step::journal::Journal;
use crate::cli::ports::Text;

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
        check: Check,
        refactor: Option<&PullRequest>,
        changes: &'change [Change],
    ) -> Result<Owed<'change>, Failure> {
        let flagged = check.flagged(changes);
        match (check, refactor) {
            (Check::Goldens, Some(_)) => Ok(Owed::new(Fault::Refactor, flagged)),
            _ => flagged
                .into_iter()
                .map(|change| Ok(self.reaching(change)?.is_none().then_some(change)))
                .filter_map(Result::transpose)
                .collect::<Result<_, Failure>>()
                .map(|failing| Owed::new(Fault::Beyond(check), failing)),
        }
    }

    fn reaching<'change>(&self, change: &'change Change) -> Result<Option<&'change Change>, Failure> {
        let beyond: Vec<Text> = change
            .paths()
            .into_iter()
            .map(|produced| Changed::reach(self.arguments(produced)))
            .collect::<Result<_, _>>()?;
        Ok(beyond.iter().all(|listed| listed.as_ref().is_empty()).then_some(change))
    }

    fn arguments(&self, produced: &Path) -> impl Iterator<Item = OsString> {
        [OsString::from(Self::BEYOND), self.metadata.as_ref().as_os_str().to_owned(), produced.as_os_str().to_owned()]
            .into_iter()
            .chain(self.sources.clone())
    }
}
