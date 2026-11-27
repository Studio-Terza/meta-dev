mod boundary;

use std::fmt;
use std::path::{Path, PathBuf};

use console_ports::Heard;
use design_messages::PullNumber;
use dispatch_ports::{Clock, Dispatch, Ran, Shell};
use file_ports::Files;
use grammar_git::Worktrees;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::subagent::Hub;
use crate::cli::failure::Failure;
use crate::cli::request::Local;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub use boundary::Used;
use boundary::{Checkout, Landed, State};

#[derive(Debug)]
struct Releasing<'used, 'crew, D: Dispatch, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, D, C, G>,
    directory: &'used Path,
    pull: PullNumber,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Released {
    Removed,
    Untouched,
}

#[derive(strum::Display, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
enum Held {
    NoRepository,
    Unviewed,
    Unlisted,
    Unstatused,
    Dirty,
    Unpushed,
    Unremoved,
}

#[derive(Debug)]
struct Holding {
    held: Held,
    worktree: Option<PathBuf>,
}

#[derive(Debug)]
struct Kept<'used> {
    holding: Holding,
    pull: PullNumber,
    directory: &'used Path,
}

impl Used {
    pub fn heard(heard: Heard) -> Result<Used, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub(crate) fn answered<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let untouched = || Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PostToolUse))));
        self.merged().map_or_else(untouched, |pull| self.released(pull, ground, crew))
    }

    fn released<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        pull: PullNumber,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let local = ground.local();
        match (Releasing { crew, directory: self.directory(), pull }).released(&local) {
            Ok(Released::Removed) => Ok(Uttered::silent(Ok(Outcome::Released))),
            Ok(Released::Untouched) => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PostToolUse)))),
            Err(holding) => {
                let queue = local.queue().ok_or(Failure::Unplaced)?;
                ground.queued(queue, Kept { holding, pull, directory: self.directory() }).map_err(Failure::Unqueued)?;
                Ok(Uttered::silent(Ok(Outcome::Escalated)))
            }
        }
    }
}

impl<D: Dispatch, C: Clock, G: Shell> Releasing<'_, '_, D, C, G> {
    fn released(&self, local: &Local) -> Result<Released, Holding> {
        let repository = local.repository().ok_or(Holding::from(Held::NoRepository))?;
        let hub = Hub::new(self.crew, self.directory, repository);
        let landed: Landed = hub.viewed(self.pull, "state,headRefName,headRefOid").ok_or(Held::Unviewed)?;
        match landed.state() {
            State::Merged => {
                self.checkout(&landed)?.map_or(Ok(Released::Untouched), |found| self.freed(&landed, &found))
            }
            State::Open | State::Closed => Ok(Released::Untouched),
        }
    }

    fn checkout(&self, landed: &Landed) -> Result<Option<Checkout>, Holding> {
        let ran = self.git(&["worktree", "list", "--porcelain"], self.directory);
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(Held::Unlisted)?;
        let listed = Worktrees::read(printed.as_ref());
        Ok(landed.found(listed.listed().iter().map(Checkout::from)))
    }

    fn freed(&self, landed: &Landed, checkout: &Checkout) -> Result<Released, Holding> {
        let held = |held: Held| Holding { held, worktree: Some(checkout.as_ref().to_path_buf()) };
        let status = self.git(&["status", "--short"], checkout.as_ref());
        let printed = status.as_ref().and_then(Ran::succeeded).ok_or_else(|| held(Held::Unstatused))?;
        printed.as_ref().is_empty().then_some(()).ok_or_else(|| held(Held::Dirty))?;
        let pushed = landed.pushed(checkout).ok_or_else(|| held(Held::Unpushed))?;
        let path = pushed.as_ref().to_string_lossy();
        let removed = self.git(&["worktree", "remove", &path], self.directory);
        removed.as_ref().and_then(Ran::succeeded).map(|_removed| Released::Removed).ok_or_else(|| held(Held::Unremoved))
    }
}

impl From<Held> for Holding {
    fn from(held: Held) -> Holding {
        Holding { held, worktree: None }
    }
}

impl fmt::Display for Kept<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "- [worktree kept] {} after merging #{}", self.holding.held, self.pull)?;
        self.holding.worktree.iter().try_for_each(|path| write!(formatter, " at {}", path.display()))?;
        write!(formatter, ", from {}", self.directory.display())
    }
}
