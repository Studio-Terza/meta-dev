use crate::event::{Attempt, Event};
use crate::ids::{AgentId, LeaseId, Timestamp};
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
    #[serde(default)]
    tool_response: Value,
}

#[derive(Deserialize)]
struct AgentInput {
    description: String,
}

/// The one field of an Agent call's `tool_response` that meta reads. Claude Code's hooks reference
/// documents `agentId` for both a completed and an `async_launched` response.
#[derive(Deserialize)]
struct AgentOutput {
    #[serde(rename = "agentId")]
    agent: AgentId,
}

/// Only these three fields are read; anything but the canned text is new instructions.
#[derive(Deserialize)]
struct SendInput {
    #[serde(default)]
    to: Value,
    #[serde(default)]
    message: Value,
    #[serde(default)]
    summary: Value,
}

/// When a hook runs: before the tool, where it can deny it, or after it, where it can only report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Before,
    After,
}

#[derive(Debug)]
pub enum Hook {
    Settled(Accepted),
    Agent(AgentStart),
    Resume(Resume),
    Bind(Binding),
}

#[derive(Debug)]
pub struct AgentStart {
    lease: Option<LeaseId>,
}

/// A `SendMessage`: the lease and agent named by a canned continue, or why it is not one.
#[derive(Debug)]
pub struct Resume(Result<Continue, String>);

#[derive(Debug)]
struct Continue {
    lease: LeaseId,
    to: AgentId,
}

/// A finished Agent call: the lease in its description and the agent id it started, or why not.
#[derive(Debug)]
pub struct Binding(Result<(LeaseId, AgentId), String>);

impl Hook {
    const AGENT_TOOLS: [&str; 2] = ["Agent", "Task"];
    const GET_LEASE: &str =
        "Get one with meta claim <task> --agent <name>, then put its lease id in the Agent description.";
    const SEND_TOOL: &str = "SendMessage";
    const CONTINUE: &str = "continue";
    const FRESH_AGENT: &str = "new instructions need a fresh agent on a fresh lease: meta claim <task> --agent <name>. \
                               A resume is only the message continue, to the agent a used lease started, \
                               with that lease id as the summary";
    const UNBOUND: &str = "no agent recorded, so a resume on this lease stays denied";

    pub fn read(phase: Phase, input: &str) -> Self {
        match Self::tool(phase, input) {
            Ok(hook) => hook,
            Err(why) => Self::Settled(phase.broken(&format!("hook input cannot be parsed ({why})"))),
        }
    }

    fn tool(phase: Phase, input: &str) -> Result<Self, String> {
        let parsed: HookInput = serde_json::from_str(input).map_err(|error| error.to_string())?;
        let tool = parsed.tool_name.as_str();
        let agent_tool = Self::AGENT_TOOLS.contains(&tool);
        match phase {
            Phase::Before if agent_tool => {
                let AgentInput { description } = Self::input(tool, parsed.tool_input)?;
                Ok(Self::Agent(AgentStart { lease: description.parse().ok() }))
            }
            Phase::Before if tool == Self::SEND_TOOL => {
                let SendInput { to, message, summary } = Self::input(tool, parsed.tool_input)?;
                let resume = match (message.as_str(), summary.as_str()) {
                    (Some(Self::CONTINUE), Some(summary)) => summary
                        .parse()
                        .map_err(|_| format!("the summary {summary:?} is not exactly a lease id"))
                        .and_then(|lease| {
                            let to = serde_json::from_value(to).map_err(|error| format!("to: {error}"))?;
                            Ok(Continue { lease, to })
                        }),
                    (Some(Self::CONTINUE), None) => Err("the summary is not a lease id".to_owned()),
                    _ => Err(format!("the message is not exactly {:?}", Self::CONTINUE)),
                };
                Ok(Self::Resume(Resume(resume)))
            }
            Phase::After if agent_tool => {
                let AgentInput { description } = Self::input(tool, parsed.tool_input)?;
                let AgentOutput { agent } = serde_json::from_value(parsed.tool_response)
                    .map_err(|error| format!("{tool} tool_response: {error}"))?;
                let lease = description
                    .parse()
                    .map_err(|_| format!("the Agent description {description:?} is not exactly a lease id"));
                Ok(Self::Bind(Binding(lease.map(|lease| (lease, agent)))))
            }
            Phase::Before | Phase::After => Ok(Self::Settled(Self::allow(Vec::new()))),
        }
    }

    fn input<T: DeserializeOwned>(tool: &str, input: Value) -> Result<T, String> {
        serde_json::from_value(input).map_err(|error| format!("{tool} input: {error}"))
    }

    fn allow(events: Vec<Event>) -> Accepted {
        Accepted::new(events, "{}\n".to_owned())
    }
}

impl Phase {
    /// The reply when meta cannot decide: before a tool, a denial; after it, a report.
    pub fn broken(self, why: &str) -> Accepted {
        let outcome = match self {
            Self::Before => "denied",
            Self::After => "not recorded",
        };
        self.refuse(format!("{why}; {outcome}, fail closed."), Vec::new())
    }

    fn refuse(self, reason: String, events: Vec<Event>) -> Accepted {
        let reply = match self {
            Self::Before => json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            }),
            Self::After => json!({ "decision": "block", "reason": reason }),
        };
        Accepted::new(events, format!("{reply}\n"))
    }
}

impl Ledger {
    pub fn admit(mut self, start: AgentStart) -> Accepted {
        let refusal = |why: String| {
            let denied = Event::Denied { attempt: Attempt::AgentStart };
            Phase::Before.refuse(format!("{why}. {}", Hook::GET_LEASE), vec![denied])
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

    /// Records the agent a used lease started, so that only that agent can be resumed on it, and the
    /// lease's deadline counted from this start at `at`.
    pub fn bind(&self, binding: Binding, at: Timestamp) -> Accepted {
        let refusal = |why: String| Phase::After.refuse(format!("{why}; {}.", Hook::UNBOUND), Vec::new());
        match binding.0 {
            Err(why) => refusal(why),
            Ok((lease, agent)) => match self.leases.get(&lease) {
                Some(LeaseSlot::Spent(spent)) => match spent.agent() {
                    None => {
                        let deadline = spent.lease().deadline_from(at).time();
                        Hook::allow(vec![Event::Started { lease, agent, deadline }])
                    }
                    Some(earlier) => refusal(format!("{lease} already started {earlier}, not {agent}")),
                },
                Some(LeaseSlot::Unused(_)) => refusal(format!("{lease} is unused, so it started no agent")),
                None => refusal(format!("{lease} is unknown")),
            },
        }
    }

    pub fn resume(&self, resume: Resume) -> Accepted {
        let refusal = |why: String| Phase::Before.refuse(format!("{why}; denied: {}.", Hook::FRESH_AGENT), Vec::new());
        let Continue { lease, to } = match resume.0 {
            Ok(request) => request,
            Err(why) => return refusal(why),
        };
        let spent = match self.leases.get(&lease) {
            Some(LeaseSlot::Spent(spent)) => spent,
            Some(LeaseSlot::Unused(_)) => {
                return refusal(format!("{lease} is unused, so no agent of it can be resumed"));
            }
            None => return refusal(format!("{lease} is unknown")),
        };
        let task = spent.lease().task();
        match (spent.agent(), self.tasks.get(&task)) {
            (None, _) => refusal(format!("no agent is recorded as started on {lease}")),
            (Some(agent), _) if *agent != to => refusal(format!("{lease} started {agent}, not {to}")),
            (Some(_), Some(progress)) if progress.is_active() => Hook::allow(vec![Event::Resumed { lease }]),
            (Some(_), progress) => refusal(format!(
                "the task of {lease} is no longer active: {}",
                progress.map_or_else(|| format!("{task} is unknown"), ToString::to_string)
            )),
        }
    }
}
