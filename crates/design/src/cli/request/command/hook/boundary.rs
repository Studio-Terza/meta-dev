use std::collections::BTreeMap;
use std::io::Write;

use console_ports::Heard;
use newtype::Newtype;
use serde::{Deserialize, Serialize};

use crate::cli::failure::Failure;
use crate::{HookEvent, Outcome};

#[derive(Debug, Serialize)]
pub struct Settings {
    hooks: BTreeMap<HookEvent, [Matcher; 1]>,
}

#[derive(Debug, Serialize)]
struct Matcher {
    hooks: [Handler; 1],
}

#[derive(Debug, Serialize)]
struct Handler {
    #[serde(rename = "type")]
    kind: &'static str,
    command: String,
    timeout: u32,
}

#[derive(Debug, Deserialize)]
pub struct Stopping {
    #[serde(rename = "stop_hook_active")]
    continuation: Continuation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Continuing;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "bool")]
struct Continuation(Option<Continuing>);

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Decision {
    Block {
        decision: Verdict,
        reason: String,
    },
    Notice {
        #[serde(rename = "systemMessage")]
        system_message: String,
    },
}

#[derive(Debug, Serialize)]
pub enum Verdict {
    #[serde(rename = "block")]
    Block,
}

impl Settings {
    const KIND: &'static str = "command";
    const SECONDS: u32 = 3600;

    pub fn listed(events: &[HookEvent]) -> Settings {
        Settings {
            hooks: events.iter().map(|event| (*event, [Matcher { hooks: [Settings::handler(*event)] }])).collect(),
        }
    }

    fn handler(event: HookEvent) -> Handler {
        Handler {
            kind: Settings::KIND,
            command: format!("design --workspace \"$CLAUDE_PROJECT_DIR\" hook {event}"),
            timeout: Settings::SECONDS,
        }
    }

    pub fn written(&self, out: &mut impl Write) -> Result<Outcome, Failure> {
        serde_json::to_writer_pretty(&mut *out, self)?;
        writeln!(out)?;
        Ok(Outcome::Settings)
    }
}

impl Stopping {
    pub fn heard(heard: Heard) -> Result<Stopping, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn decided(&self, halt: &Failure) -> Decision {
        match self.continuation.0 {
            Some(Continuing) => Decision::Notice { system_message: halt.to_string() },
            None => Decision::Block { decision: Verdict::Block, reason: halt.to_string() },
        }
    }
}

impl From<bool> for Continuation {
    fn from(active: bool) -> Continuation {
        Continuation(active.then_some(Continuing))
    }
}

impl Decision {
    pub fn written(&self, out: &mut impl Write) -> Result<Outcome, Failure> {
        serde_json::to_writer(&mut *out, self)?;
        writeln!(out)?;
        Ok(self.outcome())
    }

    fn outcome(&self) -> Outcome {
        match self {
            Decision::Block { .. } => Outcome::Blocked,
            Decision::Notice { .. } => Outcome::Noticed,
        }
    }
}
