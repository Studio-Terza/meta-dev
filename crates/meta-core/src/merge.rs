use crate::event::{Event, Route};
use crate::ids::{PrNumber, RunCount, Sha, TaskId};
use crate::ledger::Ledger;
use crate::pull::{Outcome, Pull, PullState};
use crate::reply::{Accepted, Refusal};

#[derive(Debug)]
pub struct Merging {
    task: TaskId,
    pr: PrNumber,
    pin: Sha,
}

#[derive(Debug)]
pub struct Cleared(Landed);

#[derive(Debug)]
pub struct Landed {
    task: TaskId,
    pr: PrNumber,
    head: Sha,
    runs: RunCount,
    route: Route,
}

#[derive(Debug)]
pub enum Clearance {
    Unmerged(Cleared),
    AlreadyMerged(Landed),
}

impl Ledger {
    pub fn merge(&self, pr: PrNumber) -> Result<Merging, Refusal> {
        self.linked(pr).map(|task| Merging { task: task.id(), pr, pin: task.pin().clone() }).ok_or_else(|| {
            Refusal::because(format!(
                "{pr} is unlinked. Next: meta link <task> {} from the agent that claimed the task.",
                pr.digits()
            ))
        })
    }
}

impl Merging {
    pub fn clear(self, pull: &Pull) -> Result<Clearance, Refusal> {
        let Self { task, pr, pin } = self;
        let landed = |route| Landed { task, pr, head: pull.head().clone(), runs: pull.runs(), route };
        let relink = format!("meta link {task} {}", pr.digits());
        let moved = (*pull.head() != pin).then(|| {
            format!("head moved from pinned {pin} to {}. Next: review the new head, then {relink}.", pull.head())
        });
        match (pull.state(), moved) {
            (PullState::Merged, None) => Ok(Clearance::AlreadyMerged(landed(Route::Outside))),
            (PullState::Merged, Some(moved)) => Err(Refusal::because(format!("{pr} was merged outside meta: {moved}"))),
            (PullState::Closed, _) => Err(Refusal::because(format!("{pr} is closed. Next: reopen it, then {relink}."))),
            (PullState::Open, moved) => {
                Self::checks(pull, &relink, pr, moved).map(|()| Clearance::Unmerged(Cleared(landed(Route::Meta))))
            }
        }
    }

    fn checks(pull: &Pull, relink: &str, pr: PrNumber, moved: Option<String>) -> Result<(), Refusal> {
        let unconfigured = pull.required().is_empty().then(|| {
            format!(
                "missing check: no required checks are configured. \
                 Next: require checks on main in branch protection, then meta merge {} again.",
                pr.digits()
            )
        });
        let checks = pull.required().iter().filter_map(|name| match pull.outcome(name) {
            Some(Outcome::Green) => None,
            Some(Outcome::Red) => {
                Some(format!("red check {name} on {}. Next: fix it, push, then {relink}.", pull.head()))
            }
            Some(Outcome::Pending) | None => Some(format!(
                "missing check {name} on {}. Next: wait for CI, then meta merge {} again.",
                pull.head(),
                pr.digits()
            )),
        });
        let blockers: Vec<String> = moved.into_iter().chain(unconfigured).chain(checks).collect();
        match blockers.is_empty() {
            true => Ok(()),
            false => Err(Refusal::because(format!("{pr} cannot merge:{}", Refusal::lines(&blockers)))),
        }
    }
}

impl Cleared {
    pub fn pr(&self) -> PrNumber {
        self.0.pr
    }

    pub fn head(&self) -> &Sha {
        &self.0.head
    }

    pub fn land(self) -> Landed {
        self.0
    }
}

impl Landed {
    pub fn record(self) -> Accepted {
        let Self { task, pr, head, runs, route } = self;
        let how = match route {
            Route::Meta => "squash-merged",
            Route::Outside => "was already merged outside meta",
        };
        let text = format!("{pr} {how} at {head}; {task} done\n");
        Accepted::new(vec![Event::Merged { pr, head, runs, route }, Event::Done { task }], text)
    }
}
