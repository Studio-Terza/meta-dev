//! The Agent prompt is scanned for a lease id: a registered boundary (self/parse-boundaries.toml).
#![allow(clippy::disallowed_methods)]

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
    prompt: String,
}

#[derive(Debug)]
pub enum Hook {
    Settled(Accepted),
    Agent(AgentStart),
}

#[derive(Debug)]
pub struct AgentStart {
    lease: Option<LeaseId>,
}

impl Hook {
    const AGENT_TOOLS: [&str; 2] = ["Agent", "Task"];
    const GET_LEASE: &str = "Get one with meta claim <task> --agent <name>, then put its lease id in the Agent prompt.";

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
                let AgentInput { prompt } = Self::input(tool, parsed.tool_input)?;
                let lease = prompt
                    .split(|letter: char| !(letter.is_ascii_alphanumeric() || letter == '-'))
                    .find_map(|word| word.parse::<LeaseId>().ok());
                Ok(Self::Agent(AgentStart { lease }))
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
            return refusal("the Agent prompt carries no lease id".to_owned());
        };
        match self.leases.remove(&id) {
            Some(LeaseSlot::Unused(lease)) if self.live(&lease) => Hook::allow(vec![lease.spend().1]),
            Some(LeaseSlot::Unused(lease)) => refusal(format!("{id} is not live: {} is done", lease.task())),
            Some(LeaseSlot::Spent(_)) => refusal(format!("{id} is already used; each lease starts one agent")),
            None => refusal(format!("{id} is unknown")),
        }
    }
}
