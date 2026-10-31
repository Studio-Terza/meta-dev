use crate::brief::{lease_line, work_text};
use crate::event::Event;
use crate::ids::{AgentName, Claimant, LeaseId, Origin, TaskId, Timestamp, Title};
use crate::lease::{Deadline, LeaseRole, LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::task::Progress;

#[derive(Debug)]
pub struct Claim {
    task: TaskId,
    title: Title,
    grant: Grant,
    role: LeaseRole,
}

#[derive(Debug)]
enum Grant {
    FirstClaim { agent: AgentName, lease: LeaseId },
    FreshLease { agent: AgentName, lease: LeaseId },
    SelfClaim,
}

impl Ledger {
    pub fn claim(&self, task: TaskId, claimant: Claimant, role: LeaseRole) -> Result<Claim, Refusal> {
        let lease = LeaseId::after(self.leases.len());
        let grant = |title: &Title, grant| Ok(Claim { task, title: title.clone(), grant, role });
        match (claimant, self.tasks.get(&task), self.unused_lease(task)) {
            (Claimant::Lead, Some(Progress::Open(open)), _) => grant(open.title(), Grant::SelfClaim),
            (Claimant::Lead, Some(Progress::Claimed(claimed)), _) => Err(Refusal::because(format!(
                "{task} is already claimed by {}; a self-claim takes an open task only.",
                claimed.agent()
            ))),
            (Claimant::Agent(agent), Some(Progress::Open(open)), _) => {
                grant(open.title(), Grant::FirstClaim { agent, lease })
            }
            (Claimant::Agent(_), Some(mine), _) if mine.is_self_claimed() => Err(Refusal::because(format!(
                "{mine}: the task is self-claimed, so no agent claims it. Next: the lead finishes it, or meta open <title> for new agent work."
            ))),
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
                "{task} is already claimed and {unused} is unused ({taken}). Next: meta brief {unused} prints its brief again, then put {unused} in the Agent description; \
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
    /// The brief, and the events of the claim. An agent's lease ends at `deadline`; a self-claim has
    /// no lease and no deadline, whatever `deadline` says.
    pub fn brief(self, origin: &Origin, rules: &Rules, deadline: Option<Timestamp>) -> Accepted {
        let Self { task, title, grant, role } = self;
        let (who, lease_line, deadline_line) = match &grant {
            Grant::FirstClaim { agent, lease } | Grant::FreshLease { agent, lease } => {
                (agent.to_string(), lease_line(*lease), Deadline::of(deadline).brief_line())
            }
            Grant::SelfClaim => (
                Claimant::Lead.to_string(),
                "none: the lead works this task itself".to_owned(),
                "none: self-claims have no deadline".to_owned(),
            ),
        };
        let text = work_text(task, &title, &who, &lease_line, &deadline_line, origin, rules);
        let events = match grant {
            Grant::FirstClaim { agent, lease } => {
                vec![
                    Event::Claimed { task, agent: Claimant::Agent(agent.clone()) },
                    Event::Leased { lease, task, agent: Some(agent), deadline, role },
                ]
            }
            Grant::FreshLease { agent, lease } => {
                vec![Event::Leased { lease, task, agent: Some(agent), deadline, role }]
            }
            Grant::SelfClaim => vec![Event::Claimed { task, agent: Claimant::Lead }],
        };
        Accepted::new(events, text)
    }
}
