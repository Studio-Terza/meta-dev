mod harness;

use std::collections::BTreeMap;

use std::ops::Not;
use std::path::PathBuf;

use console_ports::{Heard, Shown};
use design_model::{Fate, Halt};
use design_runs::{Account, FailuresReported, UnreportedFailures};
use design_store::{LastSaid, Overran, Overruns, StopRow, Stopped, Tally};
use dispatch_ports::{Clock, Dispatch, Moment, Wall};
use file_ports::{Epoch, Files};
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;
use transcript_ports::{Body, SessionId};

use super::super::Ground;
use super::super::structure::Crew;
use super::answers::{Answering, Answers, Recollection, Remembered, Search};
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::{HookEvent, Outcome, Uttered};
use harness::{Harness, Tasks};
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
    #[serde(rename = "transcript_path", default)]
    transcript: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Observed<'store, S: Store> {
    store: &'store S,
    tasks: Option<Tasks>,
    started: Moment,
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

    const STALLED: &'static str =
        "Nothing is running: no background shell or agent will wake you, so the turn has stalled.";
    const OVERRUN: &'static str =
        "OVERRUN: these background tasks ran 20 minutes without finishing or writing output; check and stop them:";

    pub fn tasks<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>, now: Wall) -> Option<Tasks> {
        let harness = Harness::new(ground.files(), Epoch::new(now.into_inner()));
        self.transcript.as_deref().map(|transcript| harness.tasks(transcript))
    }

    pub fn decided<S: Store, D: Dispatch, C: Clock>(
        &self,
        crew: &Crew<'_, D, C>,
        observed: &Observed<'_, S>,
    ) -> Result<Uttered, Failure> {
        match self.continuation.0 {
            Some(Continuing) => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::Stop)))),
            None => Decision::told(observed.store, HookEvent::Stop, &[], observed.blocking(crew, self)),
        }
    }

    fn stored(&self, store: &impl Store) -> Option<Body> {
        let session = self.session.clone()?;
        store.read(LastSaid::from(session)).ignoring(Ignored::Unanswered).flatten()
    }

    fn lead(halt: Halt, fresh: &[Body]) -> Option<String> {
        match halt {
            Halt::Overrun => Some(Self::overran(fresh)),
            Halt::Stalled => Some(Self::STALLED.to_owned()),
            Halt::Live => None,
        }
    }

    fn overran(fresh: &[Body]) -> String {
        format!("{} {}", Self::OVERRUN, fresh.iter().map(ToString::to_string).collect::<Vec<String>>().join(", "))
    }
}

impl<'store, S: Store> Observed<'store, S> {
    pub fn new(store: &'store S, tasks: Option<Tasks>, started: Moment) -> Observed<'store, S> {
        Observed { store, tasks, started }
    }

    fn blocking<D: Dispatch, C: Clock>(&self, crew: &Crew<'_, D, C>, stopping: &Stopping) -> Vec<String> {
        let store = self.store;
        let last = stopping.said.clone().or_else(|| stopping.stored(store));
        let search =
            last.as_ref().and_then(|said| Answering::new(store, crew).searched(said).ignoring(Ignored::Unanswered));
        let reported: Vec<Body> = store.read(Overruns).ignoring(Ignored::Unanswered).unwrap_or_default();
        let hung = self.tasks.as_ref().map_or_default(Tasks::hung);
        let live = self.tasks.as_ref().map_or_default(Tasks::live);
        let fresh: Vec<Body> = hung.iter().filter(|task| reported.contains(task).not()).cloned().collect();
        let halt =
            fresh.first().map_or_else(|| self.tasks.as_ref().map_or(Halt::Live, Halt::from), |_first| Halt::Overrun);
        let stopped: Vec<String> = Stopping::lead(halt, &fresh)
            .into_iter()
            .chain(search.as_ref().and_then(Search::answers).map(Answers::to_string))
            .collect();
        let fate = stopped.first().map_or(Fate::Passed, |_first| Fate::Blocked);
        let ran = stopping.session.clone().map(|session| StopRow::ran(session, live, halt));
        let millis = Tally::new(crew.moment().since(self.started).max(0));
        let trailed =
            ran.map(|row| search.as_ref().map_or(row.clone(), |found| found.trailed(row)).ended(fate, millis));
        store.write(Overran::from(fresh)).ignoring(Ignored::Unanswered);
        trailed.map(|row| store.write(Stopped::from(row))).and_then(|written| written.ignoring(Ignored::Unanswered));
        stopped
    }
}

impl From<bool> for Continuation {
    fn from(active: bool) -> Continuation {
        Continuation(active.then_some(Continuing))
    }
}

impl Decision {
    const RECALLED: &'static str = "Design nodes bearing on this message (design show <id>):";

    pub fn told(
        store: &impl Store,
        event: HookEvent,
        recalled: &[Remembered],
        stopped: Vec<String>,
    ) -> Result<Uttered, Failure> {
        let unreported = store.read(UnreportedFailures)?;
        let accounts = unreported.iter().map(|failure| AsRef::<Account>::as_ref(failure).to_string());
        let nodes = recalled
            .first()
            .map(|_first| Self::RECALLED.to_owned())
            .into_iter()
            .chain(recalled.iter().map(Remembered::to_string));
        let told: Vec<String> = accounts.chain(nodes).chain(stopped).collect();
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

    fn said(&self) -> Result<Shown, Failure> {
        let json = serde_json::to_string(self).map_err(Failure::Render)?;
        Ok(Shown::said(format_args!("{json}\n")))
    }

    fn outcome(&self) -> Outcome {
        match self {
            Decision::Block { .. } => Outcome::Blocked,
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

    pub fn recalled<S: Store, D: Dispatch, C: Clock>(&self, answering: &Answering<'_, S, D, C>) -> Recollection {
        self.prompt
            .as_ref()
            .and_then(|prompt| answering.recalled(prompt).ignoring(Ignored::Unrecalled))
            .unwrap_or_default()
    }
}
