use crate::event::{Attempt, Event};
use crate::ids::LeaseId;
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::reply::Accepted;
use crate::shell::ShellCommand;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct HookInput {
    tool_name: String,
    tool_input: ToolInput,
}

#[derive(Deserialize)]
struct ToolInput {
    prompt: Option<String>,
    command: Option<String>,
}

enum Tool {
    Agent(String),
    Bash(String),
    Other,
}

enum Verdict {
    Allow(Vec<Event>),
    Deny(String, Option<Attempt>),
}

impl Ledger {
    const GET_LEASE: &str = "Get one with meta claim <task> --agent <name>, then put its lease id in the Agent prompt.";
    const USE_META: &str = "Merge with meta merge <pr>; if meta cannot do it, record why with meta escape <reason>.";

    pub fn pre_tool_use(mut self, input: &str) -> Accepted {
        let verdict = match Self::tool(input) {
            Err(why) => Verdict::Deny(format!("hook input cannot be parsed ({why}); denied, fail closed."), None),
            Ok(Tool::Agent(prompt)) => self.admit(&prompt),
            Ok(Tool::Bash(command)) => match ShellCommand::parse(&command).bypass() {
                Some(bypass) => {
                    Verdict::Deny(format!("{bypass} bypasses meta. {}", Self::USE_META), Some(Attempt::Merge))
                }
                None => Verdict::Allow(Vec::new()),
            },
            Ok(Tool::Other) => Verdict::Allow(Vec::new()),
        };
        match verdict {
            Verdict::Allow(events) => Accepted { events, text: "{}\n".to_owned() },
            Verdict::Deny(reason, attempt) => Accepted {
                events: attempt.map(|attempt| Event::Denied { attempt }).into_iter().collect(),
                text: format!("{}\n", Self::denial(&reason)),
            },
        }
    }

    fn tool(input: &str) -> Result<Tool, String> {
        let parsed: HookInput = serde_json::from_str(input).map_err(|error| error.to_string())?;
        let ToolInput { prompt, command } = parsed.tool_input;
        match parsed.tool_name.as_str() {
            "Agent" => prompt.map(Tool::Agent).ok_or_else(|| "Agent input has no prompt".to_owned()),
            "Bash" => command.map(Tool::Bash).ok_or_else(|| "Bash input has no command".to_owned()),
            _ => Ok(Tool::Other),
        }
    }

    fn admit(&mut self, prompt: &str) -> Verdict {
        let lease = prompt
            .split(|letter: char| !(letter.is_ascii_alphanumeric() || letter == '-'))
            .find_map(|word| word.parse::<LeaseId>().ok());
        let refusal = |why: String| Verdict::Deny(format!("{why}. {}", Self::GET_LEASE), Some(Attempt::AgentStart));
        let Some(id) = lease else {
            return refusal("the Agent prompt carries no lease id".to_owned());
        };
        match self.leases.remove(&id) {
            Some(LeaseSlot::Unused(lease)) if self.live(&lease) => Verdict::Allow(vec![lease.spend().1]),
            Some(LeaseSlot::Unused(lease)) => refusal(format!("{id} is not live: {} is done", lease.task())),
            Some(LeaseSlot::Spent(_)) => refusal(format!("{id} is already used; each lease starts one agent")),
            None => refusal(format!("{id} is unknown")),
        }
    }

    pub fn denial(reason: &str) -> String {
        json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        })
        .to_string()
    }
}
