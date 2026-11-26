use std::collections::BTreeMap;

use console_ports::{Heard, Shown};
use design_runs::{Account, FailuresReported, UnreportedFailures};
use design_store::{LastSaid, Recalled};
use dispatch_ports::{Clock, Dispatch};
use grammar_search::Terms;
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;
use transcript_ports::{Body, SessionId};

use super::answers::{Answering, Answers};
use crate::cli::failure::Failure;
use crate::cli::output::Count;
use crate::cli::request::ignored::Ignored;
use crate::{HookEvent, Outcome, Uttered};
use invariants::Ignoring;

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
pub struct Prompting {
    prompt: Option<Body>,
}

#[derive(Debug, Deserialize)]
pub struct Stopping {
    #[serde(rename = "stop_hook_active")]
    continuation: Continuation,
    #[serde(rename = "last_assistant_message", default)]
    said: Option<Body>,
    #[serde(rename = "session_id", default)]
    session: Option<SessionId>,
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
    Context {
        #[serde(rename = "hookSpecificOutput")]
        specific: Specific,
    },
}

#[derive(Debug, Serialize)]
pub struct Specific {
    #[serde(rename = "hookEventName")]
    event: HookEvent,
    #[serde(rename = "additionalContext")]
    context: String,
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

    pub fn written(&self) -> Result<Uttered, Failure> {
        let json = serde_json::to_string_pretty(self).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Settings)))
    }
}

impl Stopping {
    pub fn heard(heard: Heard) -> Result<Stopping, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn decided<S: Store, D: Dispatch, C: Clock>(
        &self,
        concluded: Result<Count, Failure>,
        store: &S,
        answering: &Answering<'_, S, D, C>,
    ) -> Result<Uttered, Failure> {
        match self.continuation.0 {
            Some(Continuing) => Self::continued(concluded),
            None => Decision::told(store, HookEvent::Stop, &[], self.answered(store, answering).as_ref()),
        }
    }

    fn answered<S: Store, D: Dispatch, C: Clock>(
        &self,
        store: &S,
        answering: &Answering<'_, S, D, C>,
    ) -> Option<Answers> {
        let last = self.said.clone().or_else(|| self.stored(store));
        last.and_then(|said| answering.answered(&said).ignoring(Ignored::Unanswered)).flatten()
    }

    fn stored(&self, store: &impl Store) -> Option<Body> {
        let session = self.session.clone()?;
        store.read(LastSaid::from(session)).ignoring(Ignored::Unanswered).flatten()
    }

    fn continued(concluded: Result<Count, Failure>) -> Result<Uttered, Failure> {
        match concluded {
            Ok(_jobs) => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::Stop)))),
            Err(halt) => Decision::Notice { system_message: halt.to_string() }.written(),
        }
    }
}

impl From<bool> for Continuation {
    fn from(active: bool) -> Continuation {
        Continuation(active.then_some(Continuing))
    }
}

impl Decision {
    const RECALLED: &'static str = "Design nodes matching this message (design show <id>):";

    pub fn told(
        store: &impl Store,
        event: HookEvent,
        recalled: &[Recalled],
        answers: Option<&Answers>,
    ) -> Result<Uttered, Failure> {
        let unreported = store.read(UnreportedFailures)?;
        let accounts = unreported.iter().map(|failure| AsRef::<Account>::as_ref(failure).to_string());
        let nodes = recalled
            .first()
            .map(|_first| Self::RECALLED.to_owned())
            .into_iter()
            .chain(recalled.iter().map(Recalled::to_string));
        let told: Vec<String> = accounts.chain(nodes).chain(answers.map(ToString::to_string)).collect();
        told.first().map(|_first| told.join("\n")).map_or(Ok(Uttered::silent(Ok(Outcome::Hooked(event)))), |joined| {
            let decision = Decision::telling(event, joined);
            let said = decision.said()?;
            let reported = store.write(unreported.into_iter().collect::<FailuresReported>());
            Ok(Uttered::new(said, reported.map(|_reported| decision.outcome()).map_err(Failure::from)))
        })
    }

    fn telling(event: HookEvent, told: String) -> Decision {
        match event {
            HookEvent::Stop => Decision::Block { decision: Verdict::Block, reason: told },
            HookEvent::UserPromptSubmit => Decision::Context { specific: Specific { event, context: told } },
        }
    }

    fn written(&self) -> Result<Uttered, Failure> {
        Ok(Uttered::new(self.said()?, Ok(self.outcome())))
    }

    fn said(&self) -> Result<Shown, Failure> {
        let json = serde_json::to_string(self).map_err(Failure::Render)?;
        Ok(Shown::said(format_args!("{json}\n")))
    }

    fn outcome(&self) -> Outcome {
        match self {
            Decision::Block { .. } => Outcome::Blocked,
            Decision::Notice { .. } => Outcome::Noticed,
            Decision::Context { .. } => Outcome::Told,
        }
    }
}

impl Prompting {
    pub fn heard(heard: Heard) -> Result<Prompting, Failure> {
        let text = heard.into_inner();
        text.is_empty()
            .then_some(Prompting { prompt: None })
            .map_or_else(|| serde_json::from_str(&text).map_err(Failure::Unheard), Ok)
    }

    pub fn terms(&self) -> Option<Terms> {
        self.prompt.as_ref().map(Body::to_string).and_then(|prompt| Terms::lexed([prompt.as_str()]))
    }
}
