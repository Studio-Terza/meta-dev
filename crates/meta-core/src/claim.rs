use crate::event::Event;
use crate::ids::{AgentName, Claimant, LeaseId, Origin, TaskId, Title};
use crate::lease::{LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::task::Progress;

#[derive(Debug)]
pub struct Claim {
    task: TaskId,
    title: Title,
    grant: Grant,
}

#[derive(Debug)]
enum Grant {
    FirstClaim { agent: AgentName, lease: LeaseId },
    FreshLease { agent: AgentName, lease: LeaseId },
    SelfClaim,
}

impl Ledger {
    pub fn claim(&self, task: TaskId, claimant: Claimant) -> Result<Claim, Refusal> {
        let lease = LeaseId::after(self.leases.len());
        let grant = |title: &Title, grant| Ok(Claim { task, title: title.clone(), grant });
        match (claimant, self.tasks.get(&task), self.unused_lease(task)) {
            (Claimant::Lead, Some(Progress::Open(open)), _) => grant(open.title(), Grant::SelfClaim),
            (Claimant::Lead, Some(Progress::Claimed(claimed)), _) => Err(Refusal::because(format!(
                "{task} is already claimed by {}; a self-claim takes an open task only.",
                claimed.agent()
            ))),
            (Claimant::Agent(agent), Some(Progress::Open(open)), _) => {
                grant(open.title(), Grant::FirstClaim { agent, lease })
            }
            (Claimant::Agent(agent), Some(Progress::Claimed(claimed)), None) => {
                grant(claimed.title(), Grant::FreshLease { agent, lease })
            }
            (Claimant::Agent(agent), Some(Progress::Linked(linked)), None) => {
                grant(linked.title(), Grant::FreshLease { agent, lease })
            }
            (_, Some(Progress::Done(_)), _) => {
                Err(Refusal::because(format!("{task} is done. Next: meta open <title>.")))
            }
            (_, Some(gone @ Progress::Retired(_)), _) => {
                Err(Refusal::because(format!("{gone}. Next: meta open <title> for new work.")))
            }
            (_, Some(deferred @ Progress::Deferred(_)), _) => {
                Err(Refusal::because(format!("{deferred}. Next: meta undefer {task}, then claim it.")))
            }
            (Claimant::Lead, Some(linked @ Progress::Linked(_)), _) => {
                Err(Refusal::because(format!("{linked}; a self-claim takes an open task only.")))
            }
            (Claimant::Agent(_), Some(taken), Some(unused)) => Err(Refusal::because(format!(
                "{task} is already claimed and {unused} is unused ({taken}). Next: put {unused} in the Agent description; \
                 once it is spent, meta claim {task} --agent <name> issues a fresh lease."
            ))),
            (_, None, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status lists open tasks."))),
        }
    }

    fn unused_lease(&self, task: TaskId) -> Option<LeaseId> {
        self.leases.iter().find_map(|(id, slot)| match slot {
            LeaseSlot::Unused(lease) if lease.task() == task && *lease.purpose() == Purpose::Work => Some(*id),
            _ => None,
        })
    }
}

impl Claim {
    pub fn brief(self, origin: &Origin, rules: &Rules) -> Accepted {
        let Self { task, title, grant } = self;
        let (who, lease_line) = match &grant {
            Grant::FirstClaim { agent, lease } | Grant::FreshLease { agent, lease } => {
                (agent.to_string(), format!("{lease} (put it in the Agent description; it works once)"))
            }
            Grant::SelfClaim => (Claimant::Lead.to_string(), "none: the lead works this task itself".to_owned()),
        };
        let text = format!(
            "task: {task}: {title}\n\
             agent: {who}\n\
             lease: {lease_line}\n\
             setup:\n  \
             1. git clone {origin} {task}\n  \
             2. cd {task} && git switch -c {task} origin/main\n  \
             3. set the git identity that rule identity names\n  \
             4. push, open the PR with gh pr create, then run meta link {task} <pr>\n\
             rules:\n{rules}"
        );
        let events = match grant {
            Grant::FirstClaim { agent, lease } => {
                vec![Event::Claimed { task, agent: Claimant::Agent(agent) }, Event::Leased { lease, task }]
            }
            Grant::FreshLease { lease, .. } => vec![Event::Leased { lease, task }],
            Grant::SelfClaim => vec![Event::Claimed { task, agent: Claimant::Lead }],
        };
        Accepted::new(events, text)
    }
}
