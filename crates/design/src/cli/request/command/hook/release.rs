mod boundary;
mod install;

use std::path::{Path, PathBuf};

use console_ports::Environment;
use design_messages::PullNumber;
use dispatch_ports::{Clock, Ran, Shell, Timeout};
use file_ports::Files;
use grammar_git::Worktrees;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::reply::{Hook, Reply};
use super::subagent::Hub;
use crate::cli::failure::Failure;
use crate::cli::request::{Held, InstallStep, Item, Local, Queue};
use crate::{HookEvent, Outcome};
pub use boundary::Used;
use boundary::{Checkout, Landed, State};
use install::Installing;

#[derive(Debug)]
struct Releasing<'used, 'crew, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, C, G>,
    directory: &'used Path,
    pull: PullNumber,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Released {
    Removed,
    Untouched,
}

#[derive(Debug)]
enum Landing {
    Unmerged,
    Unviewed(Holding),
    Merged { released: Result<Released, Holding>, installed: Result<(), InstallStep> },
}

#[derive(Debug)]
struct Holding {
    held: Held,
    worktree: Option<PathBuf>,
}

impl Hook for Used {
    fn answered<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        &self,
        _environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Reply<'_>, Failure> {
        let untouched = || Ok(Reply::Pass(Outcome::Hooked(HookEvent::PostToolUse)));
        self.merged().map_or_else(untouched, |pull| self.released(pull, ground, crew))
    }
}

impl Used {
    fn released<F: Files, S: Store, C: Clock, G: Shell>(
        &self,
        pull: PullNumber,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Reply<'static>, Failure> {
        let local = ground.local();
        let releasing = Releasing { crew, directory: self.directory(), pull };
        let landing = match releasing.landed(&local) {
            Ok(landed) => match landed.state() {
                State::Merged => Landing::Merged {
                    released: releasing.released(&landed),
                    installed: Installing::new(crew, self.directory(), pull).installed(ground),
                },
                State::Open | State::Closed => Landing::Unmerged,
            },
            Err(holding) => Landing::Unviewed(holding),
        };
        let escalated = landing.escalated(self.directory(), pull);
        match local.queue() {
            Some(file) => {
                let read = ground.queue(file)?;
                let queue = escalated.iter().cloned().fold(landing.cleared(read.clone()), Queue::with);
                (queue != read).then(|| ground.queued(file, &queue)).transpose()?;
                Ok(())
            }
            None => escalated.first().map_or(Ok(()), |_escalated| Err(Failure::Unplaced)),
        }?;
        Ok(Reply::Pass(landing.outcome()))
    }
}

impl Landing {
    fn escalated(&self, directory: &Path, pull: PullNumber) -> Vec<Item> {
        let held = |holding: &Holding| {
            let worktree = holding.worktree.clone().unwrap_or_else(|| directory.to_path_buf());
            Item::kept(worktree, holding.held, pull.into_inner())
        };
        match self {
            Landing::Unmerged => Vec::new(),
            Landing::Unviewed(holding) => vec![held(holding)],
            Landing::Merged { released, installed } => {
                let kept = match released {
                    Ok(_released) => None,
                    Err(holding) => Some(held(holding)),
                };
                let uninstalled = match installed {
                    Ok(()) => None,
                    Err(step) => Some(Item::uninstalled(pull.into_inner(), *step)),
                };
                kept.into_iter().chain(uninstalled).collect()
            }
        }
    }

    fn cleared(&self, queue: Queue) -> Queue {
        match self {
            Landing::Merged { installed: Ok(()), .. } => queue.installed(),
            Landing::Merged { installed: Err(_), .. } | Landing::Unmerged | Landing::Unviewed(_) => queue,
        }
    }

    fn outcome(&self) -> Outcome {
        match self {
            Landing::Unmerged => Outcome::Hooked(HookEvent::PostToolUse),
            Landing::Unviewed(_)
            | Landing::Merged { released: Err(_), .. }
            | Landing::Merged { installed: Err(_), .. } => Outcome::Escalated,
            Landing::Merged { released: Ok(Released::Removed), installed: Ok(()) } => Outcome::Released,
            Landing::Merged { released: Ok(Released::Untouched), installed: Ok(()) } => Outcome::Installed,
        }
    }
}

impl<C: Clock, G: Shell> Releasing<'_, '_, C, G> {
    const PROGRAM: &'static str = "git";
    const QUICK: Timeout = Timeout::new(60);

    fn landed(&self, local: &Local) -> Result<Landed, Holding> {
        let repository = local.repository().ok_or(Holding::from(Held::NoRepository))?;
        let hub = Hub::new(self.crew, self.directory, repository);
        Ok(hub.viewed(self.pull, "state,headRefName,headRefOid", Held::Unviewed)?)
    }

    fn released(&self, landed: &Landed) -> Result<Released, Holding> {
        self.checkout(landed)?.map_or(Ok(Released::Untouched), |found| self.freed(landed, &found))
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

    fn git(&self, arguments: &[&str], directory: &Path) -> Option<Ran> {
        self.crew.spoke(Self::PROGRAM, arguments, directory, Self::QUICK)
    }
}

impl From<Held> for Holding {
    fn from(held: Held) -> Holding {
        Holding { held, worktree: None }
    }
}
