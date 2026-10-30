use crate::event::Event;
use crate::ids::{AgentName, LeaseId, Origin, TaskId, Title};
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
}

impl Ledger {
    pub fn claim(&self, task: TaskId, agent: AgentName) -> Result<Claim, Refusal> {
        match self.tasks.get(&task) {
            Some(Progress::Open(open)) => {
                Ok(Claim { task, title: open.title().clone(), agent, lease: LeaseId::after(self.leases.len()) })
            }
            Some(Progress::Done(_)) => Err(Refusal::because(format!("{task} is done. Next: meta open <title>."))),
            Some(taken) => Err(Refusal::because(format!("{task} is already claimed ({taken}). Next: meta status."))),
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status lists open tasks."))),
        }
    }
}

impl Claim {
    pub fn brief(self, origin: &Origin, rules: &Rules) -> Accepted {
        let Self { task, title, agent, lease } = self;
        let text = format!(
            "task: {task}: {title}\n\
             agent: {agent}\n\
             lease: {lease} (put it in the Agent prompt; it works once)\n\
             setup:\n  \
             1. git clone {origin} {task}\n  \
             2. cd {task} && git switch -c {task} origin/main\n  \
             3. set the git identity that rule identity names\n  \
             4. push, open the PR with gh pr create, then run meta link {task} <pr>\n\
             rules:\n{rules}"
        );
        Accepted { events: vec![Event::Claimed { task, agent }, Event::Leased { lease, task }], text }
    }
}
