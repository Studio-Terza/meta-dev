use console_ports::Shown;
use invariants::Ignoring;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use transcript_ports::Body;

use super::super::answers::Answers;
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub(crate) struct Brief {
    #[serde(flatten)]
    said: Said,
    #[serde(flatten)]
    rest: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Said {
    Prompt(Body),
    Message(Message),
}

#[derive(Debug, Deserialize)]
#[serde(from = "Value")]
struct Message(Option<Body>);

#[derive(Debug, Serialize)]
struct Rewrite<'brief> {
    #[serde(rename = "hookSpecificOutput")]
    output: Rewritten<'brief>,
}

#[derive(Debug, Serialize)]
struct Rewritten<'brief> {
    #[serde(rename = "hookEventName")]
    event: HookEvent,
    #[serde(rename = "updatedInput")]
    input: Told<'brief>,
}

#[derive(Debug, Serialize)]
struct Told<'brief> {
    #[serde(flatten)]
    said: Telling,
    #[serde(flatten)]
    rest: &'brief Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Telling {
    Prompt(String),
    Message(String),
}

impl Brief {
    pub(crate) fn body(&self) -> Option<&Body> {
        match &self.said {
            Said::Prompt(body) => Some(body),
            Said::Message(Message(body)) => body.as_ref(),
        }
    }

    pub(crate) fn rewritten(&self, body: &Body, found: &Answers) -> Result<Uttered, Failure> {
        let said = self.said.telling(format!("{body}\n\n{found}"));
        let rewrite =
            Rewrite { output: Rewritten { event: HookEvent::PreToolUse, input: Told { said, rest: &self.rest } } };
        let json = serde_json::to_string(&rewrite).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Briefed)))
    }
}

impl Said {
    fn telling(&self, text: String) -> Telling {
        match self {
            Said::Prompt(_) => Telling::Prompt(text),
            Said::Message(_) => Telling::Message(text),
        }
    }
}

impl From<Value> for Message {
    fn from(message: Value) -> Message {
        Message(serde_json::from_value(message).ignoring(Ignored::Structured))
    }
}
