use crate::event::Event;
use crate::ids::{AgentName, LeaseId, Origin, TaskId, Title};
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::task::Progress;

#[derive(Debug)]
pub struct Claim {
    task: TaskId,
    title: Title,
    agent: AgentName,
    lease: LeaseId,
    grant: Grant,
}

#[derive(Debug, Clone, Copy)]
enum Grant {
    FirstClaim,
    FreshLease,
}

impl Ledger {
    pub fn claim(&self, task: TaskId, agent: AgentName) -> Result<Claim, Refusal> {
        let lease = LeaseId::after(self.leases.len());
        let grant = |title: &Title, grant| Ok(Claim { task, title: title.clone(), agent: agent.clone(), lease, grant });
        match (self.tasks.get(&task), self.unused_lease(task)) {
            (Some(Progress::Open(open)), _) => grant(open.title(), Grant::FirstClaim),
            (Some(Progress::Claimed(claimed)), None) => grant(claimed.title(), Grant::FreshLease),
            (Some(Progress::Linked(linked)), None) => grant(linked.title(), Grant::FreshLease),
            (Some(Progress::Done(_)), _) => Err(Refusal::because(format!("{task} is done. Next: meta open <title>."))),
            (Some(taken), Some(unused)) => Err(Refusal::because(format!(
                "{task} is already claimed and {unused} is unused ({taken}). Next: put {unused} in the Agent description; \
                 once it is spent, meta claim {task} --agent <name> issues a fresh lease."
            ))),
            (None, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status lists open tasks."))),
        }
    }

    fn unused_lease(&self, task: TaskId) -> Option<LeaseId> {
        self.leases.iter().find_map(|(id, slot)| match slot {
            LeaseSlot::Unused(lease) if lease.task() == task => Some(*id),
            _ => None,
        })
    }
}

impl Claim {
    pub fn brief(self, origin: &Origin, rules: &Rules) -> Accepted {
        let Self { task, title, agent, lease, grant } = self;
        let text = format!(
            "task: {task}: {title}\n\
             agent: {agent}\n\
             lease: {lease} (put it in the Agent description; it works once)\n\
             setup:\n  \
             1. git clone {origin} {task}\n  \
             2. cd {task} && git switch -c {task} origin/main\n  \
             3. set the git identity that rule identity names\n  \
             4. push, open the PR with gh pr create, then run meta link {task} <pr>\n\
             rules:\n{rules}"
        );
        let leased = Event::Leased { lease, task };
        let events = match grant {
            Grant::FirstClaim => vec![Event::Claimed { task, agent }, leased],
            Grant::FreshLease => vec![leased],
        };
        Accepted::new(events, text)
    }
}
