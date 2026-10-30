use crate::event::{Attempt, Event};
use crate::ids::LeaseId;
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::reply::Accepted;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct HookInput {
    tool_name: String,
    #[serde(default)]
    tool_input: Value,
}

#[derive(Deserialize)]
struct AgentInput {
    description: String,
}

/// Only these two fields are read; anything but the canned text is new instructions.
#[derive(Deserialize)]
struct SendInput {
    #[serde(default)]
    message: Value,
    #[serde(default)]
    summary: Value,
}

#[derive(Debug)]
pub enum Hook {
    Settled(Accepted),
    Agent(AgentStart),
    Resume(Resume),
}

#[derive(Debug)]
pub struct AgentStart {
    lease: Option<LeaseId>,
}

/// A `SendMessage`: the lease named by a canned continue, or why it is not one.
#[derive(Debug)]
pub struct Resume(Result<LeaseId, String>);

impl Hook {
    const AGENT_TOOLS: [&str; 2] = ["Agent", "Task"];
    const GET_LEASE: &str =
        "Get one with meta claim <task> --agent <name>, then put its lease id in the Agent description.";
    const SEND_TOOL: &str = "SendMessage";
    const CONTINUE: &str = "continue";
    const FRESH_AGENT: &str = "new instructions need a fresh agent on a fresh lease: meta claim <task> --agent <name>. \
                               A resume is only the message continue with a used lease id as the summary";

    pub fn read(input: &str) -> Self {
        match Self::tool(input) {
            Ok(hook) => hook,
            Err(why) => Self::Settled(Self::broken(&format!("hook input cannot be parsed ({why})"))),
        }
    }

    pub fn broken(why: &str) -> Accepted {
        Self::deny(format!("{why}; denied, fail closed."), Vec::new())
    }

    fn tool(input: &str) -> Result<Self, String> {
        let parsed: HookInput = serde_json::from_str(input).map_err(|error| error.to_string())?;
        let tool = parsed.tool_name.as_str();
        match tool {
            _ if Self::AGENT_TOOLS.contains(&tool) => {
                let AgentInput { description } = Self::input(tool, parsed.tool_input)?;
                let lease = description.parse().ok();
                Ok(Self::Agent(AgentStart { lease }))
            }
            Self::SEND_TOOL => {
                let SendInput { message, summary } = Self::input(tool, parsed.tool_input)?;
                let lease = match (message.as_str(), summary.as_str()) {
                    (Some(Self::CONTINUE), Some(summary)) => {
                        summary.parse().map_err(|_| format!("the summary {summary:?} is not exactly a lease id"))
                    }
                    (Some(Self::CONTINUE), None) => Err("the summary is not a lease id".to_owned()),
                    _ => Err(format!("the message is not exactly {:?}", Self::CONTINUE)),
                };
                Ok(Self::Resume(Resume(lease)))
            }
            _ => Ok(Self::Settled(Self::allow(Vec::new()))),
        }
    }

    fn input<T: DeserializeOwned>(tool: &str, input: Value) -> Result<T, String> {
        serde_json::from_value(input).map_err(|error| format!("{tool} input: {error}"))
    }

    fn allow(events: Vec<Event>) -> Accepted {
        Accepted::new(events, "{}\n".to_owned())
    }

    fn deny(reason: String, events: Vec<Event>) -> Accepted {
        let reply = json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        });
        Accepted::new(events, format!("{reply}\n"))
    }
}

impl Ledger {
    pub fn admit(mut self, start: AgentStart) -> Accepted {
        let refusal = |why: String| {
            let denied = Event::Denied { attempt: Attempt::AgentStart };
            Hook::deny(format!("{why}. {}", Hook::GET_LEASE), vec![denied])
        };
        let Some(id) = start.lease else {
            return refusal("the Agent description is not exactly a lease id".to_owned());
        };
        match self.leases.remove(&id) {
            Some(LeaseSlot::Unused(lease)) if self.live(&lease) => Hook::allow(vec![lease.spend().1]),
            Some(LeaseSlot::Unused(lease)) => refusal(format!("{id} is not live: {} is done", lease.task())),
            Some(LeaseSlot::Spent(_)) => refusal(format!("{id} is already used; each lease starts one agent")),
            None => refusal(format!("{id} is unknown")),
        }
    }

    pub fn resume(&self, resume: Resume) -> Accepted {
        let refusal = |why: String| Hook::deny(format!("{why}; denied: {}.", Hook::FRESH_AGENT), Vec::new());
        match resume.0 {
            Err(why) => refusal(why),
            Ok(lease) => match self.leases.get(&lease) {
                Some(LeaseSlot::Spent(_)) => Hook::allow(vec![Event::Resumed { lease }]),
                Some(LeaseSlot::Unused(_)) => refusal(format!("{lease} is unused, so no agent of it can be resumed")),
                None => refusal(format!("{lease} is unknown")),
            },
        }
    }
}
