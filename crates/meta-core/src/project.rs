use crate::event::Event;
use crate::ids::{PrNumber, TaskId, Timestamp, Verdict};
use crate::lease::{LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::merge::Clearance;
use crate::merges::Unreached;
use crate::pull::Pull;
use crate::reply::{Accepted, Refusal};
use crate::role::Role;
use crate::status::section;
use crate::task::{Linked, Progress, Task};
use std::collections::{BTreeMap, BTreeSet};

/// Each linked PR of a project, as GitHub reports it or why it could not.
pub type ProjectPulls = BTreeMap<PrNumber, Result<Pull, Unreached>>;

impl Ledger {
    /// The project `task` is a member of. The root is not a member of its own project.
    pub(crate) fn project_of(&self, task: TaskId) -> Option<TaskId> {
        self.projects.iter().find_map(|(root, members)| members.contains(&task).then_some(*root))
    }

    fn members(&self, project: TaskId) -> Result<&BTreeSet<TaskId>, Refusal> {
        self.projects
            .get(&project)
            .ok_or_else(|| Refusal::because(format!("{project} is not a project. Next: meta role {project} project.")))
    }

    /// Joins `task` to `project`, for good: a task has one project.
    pub fn join(&self, project: TaskId, task: TaskId) -> Result<Accepted, Refusal> {
        self.members(project)?;
        match (self.tasks.get(&task), self.role_of(task), self.project_of(task)) {
            (None, _, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
            (Some(_), Some(role), _) => {
                Err(Refusal::because(format!("{task} is marked {role}, so it joins no project.")))
            }
            (Some(_), None, Some(held)) => Err(Refusal::because(format!("{task} is already a member of {held}."))),
            (Some(_), None, None) => {
                Ok(Accepted::new(vec![Event::Joined { task, project }], format!("{task} joined {project}\n")))
            }
        }
    }

    /// The root first, then the members: the order the view lists them in.
    fn project_tasks(&self, project: TaskId) -> Result<Vec<TaskId>, Refusal> {
        let members = self.members(project)?;
        Ok(std::iter::once(project).chain(members.iter().copied()).collect())
    }

    /// The PRs `meta project` asks GitHub about.
    pub fn project_prs(&self, project: TaskId) -> Result<Vec<PrNumber>, Refusal> {
        Ok(self.project_tasks(project)?.into_iter().filter_map(|task| self.linked_pr_of(task)).collect())
    }

    /// What a project agent needs to go on: who holds the project, and each task's next step.
    pub fn project(&self, project: TaskId, pulls: &ProjectPulls, at: Timestamp) -> Result<Accepted, Refusal> {
        let tasks: Vec<String> = self
            .project_tasks(project)?
            .into_iter()
            .filter_map(|task| self.tasks.get(&task).map(|progress| (task, progress)))
            .flat_map(|(task, progress)| {
                let step = self.next_step(task, progress, pulls).map(|step| format!("  next: {step}"));
                std::iter::once(progress.to_string()).chain(step)
            })
            .collect();
        let text = [
            format!("project {project}\n"),
            section("agent", &[self.project_agent(project, at)]),
            section("tasks", &tasks),
        ]
        .concat();
        Ok(Accepted::new(Vec::new(), text))
    }

    /// The latest work lease on the root, which is the project's agent or the one to replace it.
    fn project_agent(&self, project: TaskId, at: Timestamp) -> String {
        let latest = self
            .leases
            .iter()
            .rev()
            .find(|(_, slot)| slot.lease().task() == project && *slot.lease().purpose() == Purpose::Work);
        match latest {
            None => format!("none. Next: meta claim {project} --agent <name>"),
            Some((id, slot @ LeaseSlot::Unused(_))) => format!("{id}: unused, {}", slot.deadline()),
            Some((id, slot)) => {
                let seen = slot.last_seen(at).unwrap_or_else(|| "no turn end recorded".to_owned());
                format!("{id}: {}, {}, {seen}", slot.holder(), slot.deadline())
            }
        }
    }

    fn next_step(&self, task: TaskId, progress: &Progress, pulls: &ProjectPulls) -> Option<String> {
        match (self.waiting.get(&task), progress) {
            (_, Progress::Done(_) | Progress::Retired(_) | Progress::Deferred(_)) => None,
            (Some(waiting), _) => Some(format!("waiting on the owner: {}", waiting.question())),
            (None, Progress::Open(_)) => Some(format!("claim it: meta claim {task} --self")),
            (None, Progress::Claimed(_)) => Some(format!("no PR yet: push it, then meta link {task} <pr>")),
            (None, Progress::Linked(linked)) => Some(self.linked_step(linked, pulls)),
        }
    }

    /// A linked task's next step. Only the pinned head counts: a verdict on any other says nothing.
    fn linked_step(&self, linked: &Task<Linked>, pulls: &ProjectPulls) -> String {
        let (task, pr, pin) = (linked.id(), linked.pr(), linked.pin());
        let digits = pr.digits();
        let pull = match pulls.get(&pr) {
            Some(Ok(pull)) => pull,
            Some(Err(why)) => return format!("{pr} cannot be read: {why}"),
            None => return format!("{pr} was not read"),
        };
        if !pull.is_at(pin) {
            return format!("{pr} moved from pinned {pin} to {}: meta link {task} {digits}", pull.head());
        }
        if let Some(review) = self.review_under_way(linked) {
            return review;
        }
        match self.judged_head(linked) {
            None => format!("no verdict at {pin}: meta review {task} --agent <name>, then meta worker start <lease>"),
            Some((judged, Verdict::Fail)) => {
                let note = self.leases.get(&judged).map_or_else(String::new, LeaseSlot::worker_note);
                format!("fail at {pin} by {judged}{note}: judge its findings, fix, push, meta link {task} {digits}")
            }
            Some((_, Verdict::Pass)) => match self.merge(pr).and_then(|merging| merging.clear(pull)) {
                Ok(Clearance::Unmerged(_)) => {
                    format!("ready: pass at {pin}, gates green. The lead runs meta merge {digits}")
                }
                Ok(Clearance::AlreadyMerged(_)) => format!("merged at {pin}: meta merge {digits} records it"),
                Err(refusal) => format!("pass at {pin}, but {refusal}"),
            },
        }
    }

    /// The review of the pinned head that has not ended, if there is one.
    fn review_under_way(&self, linked: &Task<Linked>) -> Option<String> {
        self.leases.iter().find_map(|(id, slot)| {
            let at_pin = matches!(
                slot.lease().purpose(),
                Purpose::Review { pr, head } if *pr == linked.pr() && head == linked.pin()
            );
            match (at_pin && !slot.is_finished(), slot) {
                (false, _) => None,
                (true, LeaseSlot::Unused(_)) => Some(format!("review {id} is unused: meta worker start {id}")),
                (true, LeaseSlot::Spent(_)) => {
                    Some(format!("review {id} is under way, {}{}", slot.deadline(), slot.worker_note()))
                }
            }
        })
    }

    /// Whether the task's role says its agent runs through every active stage.
    pub(crate) fn is_project(&self, task: TaskId) -> bool {
        self.role_of(task) == Some(Role::Project)
    }
}
