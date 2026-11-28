use console_ports::Shown;
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use invariants::Ignoring;
use json_ports::{Fields, Json, Lenient};
use store_ports::Store;
use transcript_ports::Body;

use super::super::Ground;
use super::super::crew::Crew;
use super::super::ingest::Ingest;
use super::answers::{Answering, Moment};
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, serde::Deserialize)]
pub struct Brief {
    #[serde(flatten)]
    said: Said,
    #[serde(flatten)]
    rest: Fields,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Said {
    Prompt(Body),
    Message(Lenient<Body>),
}

#[derive(Debug, serde::Serialize)]
struct Rewrite<'brief> {
    #[serde(rename = "hookSpecificOutput")]
    output: Rewritten<'brief>,
}

#[derive(Debug, serde::Serialize)]
struct Rewritten<'brief> {
    #[serde(rename = "hookEventName")]
    event: HookEvent,
    #[serde(rename = "updatedInput")]
    input: Told<'brief>,
}

#[derive(Debug, serde::Serialize)]
struct Told<'brief> {
    #[serde(flatten)]
    said: Telling,
    #[serde(flatten)]
    rest: &'brief Fields,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum Telling {
    Prompt(String),
    Message(String),
}

impl Brief {
    pub fn told<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let passed = || Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse))));
        self.said.body().map_or_else(passed, |body| {
            let store = ground.store();
            ground.prepared(crew.scratch())?;
            Ingest::read(store, &ground.transcripts())?;
            let answers = Answering::new(store, crew).searched(body, Moment::Briefing).ignoring(Ignored::Unanswered);
            answers
                .flatten()
                .map_or_else(passed, |found| self.rewritten(self.said.telling(format!("{body}\n\n{found}"))))
        })
    }

    fn rewritten(&self, said: Telling) -> Result<Uttered, Failure> {
        let rewrite =
            Rewrite { output: Rewritten { event: HookEvent::PreToolUse, input: Told { said, rest: &self.rest } } };
        let json = Json::written(&rewrite).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Briefed)))
    }
}

impl Said {
    fn body(&self) -> Option<&Body> {
        match self {
            Said::Prompt(body) => Some(body),
            Said::Message(body) => body.read(),
        }
    }

    fn telling(&self, text: String) -> Telling {
        match self {
            Said::Prompt(_) => Telling::Prompt(text),
            Said::Message(_) => Telling::Message(text),
        }
    }
}
