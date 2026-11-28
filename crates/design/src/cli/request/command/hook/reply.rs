use console_ports::{Environment, Heard, Shown};
use file_ports::Files;
use newtype::Wrapping;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Tape;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

pub trait Hook: DeserializeOwned {
    fn heard(heard: Heard) -> Result<Self, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    fn answered<F: Files + Clone, S: Store>(
        &self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure>;
}

#[derive(Debug, Serialize)]
pub struct Told<'hook> {
    #[serde(flatten)]
    said: Telling,
    #[serde(flatten)]
    rest: &'hook Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Telling {
    Prompt(String),
    Message(String),
}

#[derive(Debug)]
pub enum Reply<'hook> {
    Pass(Outcome),
    Block { reason: String },
    Deny { reason: String },
    Rewrite { input: Told<'hook> },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Envelope<'reply> {
    Decided {
        decision: Decision,
        reason: &'reply str,
    },
    Specific {
        #[serde(rename = "hookSpecificOutput")]
        output: Specific<'reply>,
    },
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Decision {
    #[serde(rename = "block")]
    Block,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Specific<'reply> {
    Denied {
        #[serde(rename = "hookEventName")]
        event: HookEvent,
        #[serde(rename = "permissionDecision")]
        decision: Permission,
        #[serde(rename = "permissionDecisionReason")]
        reason: &'reply str,
    },
    Rewritten {
        #[serde(rename = "hookEventName")]
        event: HookEvent,
        #[serde(rename = "updatedInput")]
        input: &'reply Told<'reply>,
    },
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Permission {
    #[serde(rename = "deny")]
    Deny,
}

impl<'hook> Told<'hook> {
    pub fn new(said: Telling, rest: &'hook Map<String, Value>) -> Told<'hook> {
        Told { said, rest }
    }
}

impl Reply<'_> {
    pub fn uttered(&self) -> Result<Uttered, Failure> {
        self.envelope().map_or_else(
            || Ok(Uttered::silent(Ok(self.outcome()))),
            |envelope| {
                let json = serde_json::to_string(&envelope).map_err(Failure::Render)?;
                Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(self.outcome())))
            },
        )
    }

    fn outcome(&self) -> Outcome {
        match self {
            Reply::Pass(outcome) => outcome.clone(),
            Reply::Block { .. } | Reply::Deny { .. } => Outcome::Blocked,
            Reply::Rewrite { .. } => Outcome::Briefed,
        }
    }

    fn envelope(&self) -> Option<Envelope<'_>> {
        match self {
            Reply::Pass(_) => None,
            Reply::Block { reason } => Some(Envelope::Decided { decision: Decision::Block, reason }),
            Reply::Deny { reason } => Some(Envelope::Specific {
                output: Specific::Denied { event: HookEvent::PreToolUse, decision: Permission::Deny, reason },
            }),
            Reply::Rewrite { input } => {
                Some(Envelope::Specific { output: Specific::Rewritten { event: HookEvent::PreToolUse, input } })
            }
        }
    }
}
