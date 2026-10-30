use crate::event::{Attempt, Event};
use crate::ids::LeaseId;
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::reply::Accepted;
use crate::shell::ShellCommand;
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

#[derive(Deserialize)]
struct BashInput {
    command: String,
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
    const USE_META: &str = "Merge with meta merge <pr>; if meta cannot do it, record why with meta escape <reason>.";

    pub fn read(input: &str) -> Self {
        match Self::tool(input) {
            Ok(hook) => hook,
            Err(why) => Self::Settled(Self::broken(&format!("hook input cannot be parsed ({why})"))),
        }
    }

    pub fn broken(why: &str) -> Accepted {
        Self::deny(format!("{why}; denied, fail closed."), None)
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
            "Bash" => {
                let BashInput { command } = Self::input(tool, parsed.tool_input)?;
                Ok(Self::Settled(match ShellCommand::parse(&command).bypass() {
                    Some(bypass) => {
                        let next = match bypass.attempt() {
                            Attempt::AgentStart => Self::GET_LEASE,
                            Attempt::Merge => Self::USE_META,
                        };
                        Self::deny(format!("{bypass}. {next}"), Some(bypass.attempt()))
                    }
                    None => Self::allow(Vec::new()),
                }))
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

    fn deny(reason: String, attempt: Option<Attempt>) -> Accepted {
        let reply = json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        });
        Accepted::new(attempt.map(|attempt| Event::Denied { attempt }).into_iter().collect(), format!("{reply}\n"))
    }
}

impl Ledger {
    pub fn admit(mut self, start: AgentStart) -> Accepted {
        let refusal = |why: String| Hook::deny(format!("{why}. {}", Hook::GET_LEASE), Some(Attempt::AgentStart));
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
