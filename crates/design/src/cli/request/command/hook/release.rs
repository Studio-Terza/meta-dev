mod boundary;
mod install;

use std::path::{Path, PathBuf};

use design_messages::PullNumber;
use dispatch_ports::{Ran, Timeout};
use file_ports::Files;
use grammar::git::Worktrees;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::{Asked, Cut, Tape};
use super::reply::{Hook, Reply};
use super::subagent::Hub;
use crate::cli::failure::Failure;
use crate::cli::request::{Held, InstallStep, Item, Local, Queue};
use crate::{HookEvent, Outcome};
pub use boundary::Used;
use boundary::{Checkout, Landed, State};
use install::Installing;

#[derive(Debug)]
struct Releasing<'used, 'run, F: Files, S: Store> {
    directory: &'used Path,
    pull: PullNumber,
    ground: &'used Ground<'run, F, S>,
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
    fn answered<F: Files + Clone, S: Store>(
        &self,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure> {
        let untouched = || Ok(Reply::Pass(Outcome::Hooked(HookEvent::PostToolUse)));
        self.merged().map_or_else(untouched, |pull| self.released(pull, ground, tape))
    }
}

impl Used {
    fn released<F: Files, S: Store>(
        &self,
        pull: PullNumber,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'static>, Failure> {
        let local = ground.local();
        let directory = ground.resolved(self.directory());
        let releasing = Releasing { directory: &directory, pull, ground };
        let landing = match Cut::settled(releasing.landed(tape, &local))? {
            Ok(landed) => match landed.state() {
                State::Merged => Landing::Merged {
                    released: Cut::settled(releasing.released(tape, &landed))?,
                    installed: Installing::new(pull).installed(ground, tape)?,
                },
                State::Open | State::Closed => Landing::Unmerged,
            },
            Err(holding) => Landing::Unviewed(holding),
        };
        let escalated = landing.escalated(ground.rendered(&directory), pull);
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

impl<F: Files, S: Store> Releasing<'_, '_, F, S> {
    const PROGRAM: &'static str = "git";
    const QUICK: Timeout = Timeout::new(60);
    const HERE: &'static str = ".";

    fn landed(&self, tape: &Tape<'_>, local: &Local) -> Result<Landed, Cut<Holding>> {
        let repository = local.repository().ok_or(Cut::from(Held::NoRepository))?;
        let hub = Hub::new(self.directory, repository);
        hub.viewed(tape, self.pull, "state,headRefName,headRefOid", Holding::from(Held::Unviewed))
    }

    fn released(&self, tape: &Tape<'_>, landed: &Landed) -> Result<Released, Cut<Holding>> {
        self.checkout(tape, landed)?.map_or(Ok(Released::Untouched), |found| self.freed(tape, landed, &found))
    }

    fn checkout(&self, tape: &Tape<'_>, landed: &Landed) -> Result<Option<Checkout>, Cut<Holding>> {
        let ran = Self::git(tape, &["worktree", "list", "--porcelain"], self.directory)?;
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(Cut::from(Held::Unlisted))?;
        let listed = Worktrees::from(printed.as_ref());
        Ok(landed.found(listed.listed().iter().map(|worktree| Checkout::new(worktree, self.ground))))
    }

    fn freed(&self, tape: &Tape<'_>, landed: &Landed, checkout: &Checkout) -> Result<Released, Cut<Holding>> {
        let worktree = self.ground.rendered(checkout.as_ref());
        let held = |held: Held| Cut::Failed(Holding { held, worktree: Some(worktree.to_path_buf()) });
        let status = Self::git(tape, &["status", "--short"], checkout.as_ref())?;
        let printed = status.as_ref().and_then(Ran::succeeded).ok_or_else(|| held(Held::Unstatused))?;
        printed.as_ref().is_empty().then_some(()).ok_or_else(|| held(Held::Dirty))?;
        let pushed = landed.pushed(checkout).ok_or_else(|| held(Held::Unpushed))?;
        let removed = Self::git(tape, &["worktree", "remove", Self::HERE], pushed.as_ref())?;
        removed.as_ref().and_then(Ran::succeeded).map(|_removed| Released::Removed).ok_or_else(|| held(Held::Unremoved))
    }

    fn git(tape: &Tape<'_>, arguments: &[&str], directory: &Path) -> Result<Option<Ran>, Asked> {
        tape.spoke(Self::PROGRAM, arguments, directory, Self::QUICK)
    }
}

impl From<Held> for Holding {
    fn from(held: Held) -> Holding {
        Holding { held, worktree: None }
    }
}

impl From<Held> for Cut<Holding> {
    fn from(held: Held) -> Cut<Holding> {
        Cut::Failed(Holding::from(held))
    }
}
