use design_messages::OwnerHistory;
use file_ports::Files;
use invariants::Ignoring;
use serde::Deserialize;
use serde_json::{Map, Value};
use store_ports::Store;
use transcript_ports::Body;

use super::super::Ground;
use super::super::crew::Tape;
use super::super::ingest::Ingest;
use super::answers::{Answers, Moment};
use super::reply::{Reply, Telling, Told};
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub struct Brief {
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

impl Brief {
    pub fn told<F: Files + Clone, S: Store>(
        &self,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure> {
        let passed = || Ok(Reply::Pass(Outcome::Hooked(HookEvent::PreToolUse)));
        self.said.body().map_or_else(passed, |body| {
            let store = ground.store();
            ground.prepared(tape.scratch())?;
            Ingest::read(store, &ground.transcripts())?;
            let history = store.read(OwnerHistory).ignoring(Ignored::Unanswered);
            let answers = history.map(|said| Answers::searched(tape, body, Moment::Briefing, &said)).transpose()?;
            answers.flatten().map_or_else(passed, |found| {
                let said = self.said.telling(format!("{body}\n\n{found}"));
                Ok(Reply::Rewrite { input: Told::new(said, &self.rest) })
            })
        })
    }
}

impl Said {
    fn body(&self) -> Option<&Body> {
        match self {
            Said::Prompt(body) => Some(body),
            Said::Message(Message(body)) => body.as_ref(),
        }
    }

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
