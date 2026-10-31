use crate::ids::{AgentId, Binary, LeaseId, Origin, PrNumber, Sha, TaskId, Title};
use crate::lease::{Deadline, LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::task::Progress;
use crate::worker::ReviewCopy;

/// The lease line of a brief for an agent.
pub(crate) fn lease_line(lease: LeaseId) -> String {
    format!("{lease} (put it in the Agent description; it works once)")
}

/// The brief of a work lease; the claim that issued it and `meta brief` print it the same way.
pub(crate) struct WorkText<'a> {
    pub(crate) task: TaskId,
    pub(crate) title: &'a Title,
    pub(crate) who: &'a str,
    pub(crate) lease_line: &'a str,
    pub(crate) deadline_line: &'a str,
    pub(crate) origin: &'a Origin,
    pub(crate) binary: &'a Binary,
    pub(crate) rules: &'a Rules,
}

impl WorkText<'_> {
    pub(crate) fn text(&self) -> String {
        let Self { task, title, who, lease_line, deadline_line, origin, binary, rules } = self;
        format!(
            "task: {task}: {title}\n\
             agent: {who}\n\
             lease: {lease_line}\n\
             deadline: {deadline_line}\n\
             setup:\n  \
             1. git clone {origin} {task}\n  \
             2. cd {task} && git switch -c {task} origin/main\n  \
             3. set the git identity that rule identity names\n  \
             4. push, then open the PR with gh pr create\n  \
             5. from this clone, which holds the branch, run this main build, which writes the real log; build nothing: {binary} link {task} <pr>\n  \
             A meta built from your branch writes only a scratch log: run it with XDG_DATA_HOME set to a scratch dir.\n\
             rules:\n{rules}"
        )
    }
}

/// The brief of a review lease; the review that issued it and `meta brief` print it the same way.
pub(crate) struct ReviewText<'a> {
    pub(crate) pr: PrNumber,
    pub(crate) task: TaskId,
    pub(crate) title: &'a Title,
    pub(crate) agent: &'a str,
    pub(crate) lease: LeaseId,
    pub(crate) deadline_line: &'a str,
    pub(crate) head: &'a Sha,
    pub(crate) binary: &'a Binary,
    pub(crate) rules: &'a Rules,
}

impl ReviewText<'_> {
    pub(crate) fn text(&self) -> String {
        let Self { pr, task, title, agent, lease, deadline_line, head, binary, rules } = self;
        let digits = pr.digits();
        let lease_line = format!(
            "{lease} (for the lead: put it in the Agent description; it works once. Reviewer: nothing for you to do with this line)"
        );
        format!(
            "review: {pr} for {task}: {title}\n\
             agent: {agent}\n\
             lease: {lease_line}\n\
             deadline: {deadline_line}\n\
             goal: find what is wrong with {pr} at head {head}, judged against {task} and the rules below.\n\
             constraints:\n  \
             - read the change with gh pr view {digits} and gh pr diff {digits}; push, comment, approve and merge nothing\n  \
             - report each finding with its file, line and evidence, then a verdict: pass or fail\n  \
             - finish by recording the verdict yourself with this main build, which writes the real log; build nothing: \
             {binary} verdict {task} pass|fail. \
             meta refuses the verdict if {pr} no longer has head {head}; then report that and stop\n\
             rules:\n{rules}"
        )
    }
}

/// The brief of a headless worker on a review lease: meta fetches the change and records the
/// verdict, so it asks for neither `gh` nor `meta verdict`.
pub(crate) struct WorkerReviewText<'a> {
    pub(crate) pr: PrNumber,
    pub(crate) task: TaskId,
    pub(crate) title: &'a Title,
    pub(crate) agent: &'a AgentId,
    pub(crate) deadline_line: &'a str,
    pub(crate) head: &'a Sha,
    pub(crate) rules: &'a Rules,
}

impl WorkerReviewText<'_> {
    pub(crate) fn text(&self) -> String {
        let Self { pr, task, title, agent, deadline_line, head, rules } = self;
        let (view, diff) = (ReviewCopy::VIEW, ReviewCopy::DIFF);
        format!(
            "review: {pr} for {task}: {title}\n\
             agent: {agent}\n\
             deadline: {deadline_line}\n\
             goal: find what is wrong with {pr} at head {head}, judged against {task} and the rules below.\n\
             constraints:\n  \
             - the change is in this directory, as read at head {head}: {view} holds gh pr view and {diff} holds gh pr diff. \
             You have Read, Grep and Glob; nothing outside this directory can be read and no command can be run\n  \
             - give each finding with its file, line and evidence, and a verdict, pass or fail, in the structured output. \
             meta records that verdict; it reads nothing else you write\n\
             rules:\n{rules}"
        )
    }
}

/// A brief to print again for a lease that is unused and live.
#[derive(Debug)]
pub struct Reissue {
    task: TaskId,
    title: Title,
    who: String,
    lease: LeaseId,
    deadline: Deadline,
    purpose: Purpose,
}

impl Ledger {
    /// Allows `meta brief` for `lease`: unused, and its task still active.
    pub fn reissue(&self, lease: LeaseId) -> Result<Reissue, Refusal> {
        let slot = self
            .leases
            .get(&lease)
            .ok_or_else(|| Refusal::because(format!("{lease} is unknown. Next: meta status lists the live leases.")))?;
        let LeaseSlot::Unused(unused) = slot else {
            let task = slot.lease().task();
            let renew = match slot.lease().purpose() {
                Purpose::Work => format!("meta claim {task} --agent <name>"),
                Purpose::Review { .. } => format!("meta review {task} --agent <name>"),
            };
            return Err(Refusal::because(format!(
                "{lease} is already used; each lease starts one agent. Next: {renew} issues a fresh lease."
            )));
        };
        let task = unused.task();
        let progress = self.tasks.get(&task);
        let not_live = |why: String| Refusal::because(format!("{lease} is not live: {why}. Next: meta status."));
        let progress = progress.ok_or_else(|| not_live(format!("{task} is unknown")))?;
        let (title, holder) = match progress {
            Progress::Claimed(claimed) => (claimed.title(), claimed.agent()),
            Progress::Linked(linked) => (linked.title(), linked.agent()),
            other => return Err(not_live(other.to_string())),
        };
        if let (Purpose::Review { pr, .. }, Progress::Linked(linked)) = (unused.purpose(), progress)
            && linked.pr() != *pr
        {
            return Err(not_live(format!("{task} no longer links {pr}")));
        }
        let who = match (unused.agent(), unused.purpose()) {
            (Some(agent), _) => agent.to_string(),
            (None, Purpose::Work) => holder.to_string(),
            (None, Purpose::Review { .. }) => "not recorded".to_owned(),
        };
        Ok(Reissue {
            task,
            title: title.clone(),
            who,
            lease,
            deadline: slot.deadline(),
            purpose: unused.purpose().clone(),
        })
    }
}

impl Reissue {
    /// The brief of the headless worker `agent` on this review, which ends at `deadline`.
    pub(crate) fn worker_brief(
        &self,
        agent: &AgentId,
        pr: PrNumber,
        head: &Sha,
        deadline: Deadline,
        rules: &Rules,
    ) -> String {
        let deadline_line = deadline.brief_line();
        let (task, title) = (self.task, &self.title);
        WorkerReviewText { pr, task, title, agent, deadline_line: &deadline_line, head, rules }.text()
    }

    /// The brief the lease's own claim or review printed; no event.
    pub fn brief(self, origin: &Origin, binary: &Binary, rules: &Rules) -> Accepted {
        let Self { task, title, who, lease, deadline, purpose } = self;
        let deadline_line = deadline.brief_line();
        let text = match purpose {
            Purpose::Work => WorkText {
                task,
                title: &title,
                who: &who,
                lease_line: &lease_line(lease),
                deadline_line: &deadline_line,
                origin,
                binary,
                rules,
            }
            .text(),
            Purpose::Review { pr, head } => ReviewText {
                pr,
                task,
                title: &title,
                agent: &who,
                lease,
                deadline_line: &deadline_line,
                head: &head,
                binary,
                rules,
            }
            .text(),
        };
        Accepted::new(Vec::new(), text)
    }
}
