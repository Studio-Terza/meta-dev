mod boundary;
mod edits;
mod since;

use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::Lines;

use super::super::super::journal::Journal;
use super::super::kept::{Commits, Mutant, Outcomes};
use super::tested::Reach;
use crate::cli::command::failure::Failure;
use crate::cli::ignored::Ignored;
use crate::cli::ports::{Invocation, Ports, Text};
pub(super) use boundary::Choice;
use boundary::Reason;
use edits::Edits;
use invariants::Ignoring;
use since::Since;

#[derive(Debug)]
pub(super) struct Prior {
    directory: PathBuf,
    metadata: PathBuf,
}

impl Prior {
    const TESTED: &'static str = "tested";
    const DOWNLOADED: &'static str = "download";
    const OUTCOMES: &'static str = "outcomes.json";
    const HEAD: [&'static str; 2] = ["rev-parse", "HEAD^2"];

    pub(super) fn new(directory: PathBuf, metadata: PathBuf) -> Prior {
        Prior { directory, metadata }
    }

    pub(super) fn chosen(&self, ports: &impl Ports, reach: &Reach, diff: &Path) -> Result<Choice, Failure<'static>> {
        let marker = ports.files().read_text(&self.directory.join(Self::TESTED)).ignoring(Ignored::Unmarked);
        let commits = marker.as_ref().and_then(Commits::read);
        commits.map_or(Ok(Choice::Full(Reason::NoPrior)), |found| self.compared(ports, reach, &found, diff))
    }

    fn compared(
        &self,
        ports: &impl Ports,
        reach: &Reach,
        commits: &Commits,
        diff: &Path,
    ) -> Result<Choice, Failure<'static>> {
        let heads = Invocation::new("git", Self::HEAD);
        let printed = ports.output(&heads).map_err(Failure::starting(&heads))?;
        let head = Text::new(printed.as_ref().parse::<Lines>().into_iter().flatten().next().unwrap_or_default());
        let edited = Edits::new(&self.metadata, commits).found(ports, &head)?;
        let missed = self.missed(ports)?;
        let since = Since::new(&self.directory, reach, commits, diff);
        edited.map_or_else(|| since.chosen(ports, &missed), |reason| Ok(Choice::Full(reason)))
    }

    fn missed(&self, ports: &impl Ports) -> Result<Vec<Mutant>, Failure<'static>> {
        let downloads = self.directory.join(Self::DOWNLOADED);
        let shards = ports.files().entries(&downloads).map_err(|failure| Failure::unusable(downloads, failure))?;
        let journals: Vec<Journal> = shards.iter().map(|shard| Journal::from(shard.join(Self::OUTCOMES))).collect();
        let read: Vec<Text> =
            Journal::read_present(ports.files(), journals).into_iter().flatten().collect::<Result<_, _>>()?;
        let outcomes: Vec<Outcomes> =
            read.iter().map(Outcomes::read).collect::<Result<_, _>>().map_err(Failure::Outcomes)?;
        Ok(outcomes.into_iter().flat_map(Outcomes::missed).collect())
    }
}
