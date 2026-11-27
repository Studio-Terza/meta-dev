mod harness;

use std::collections::BTreeMap;
use std::fmt;

use std::ops::Not;
use std::path::PathBuf;

use console_ports::{Heard, Shown};
use design_messages::{Fate, Halt, LastSaid, Overran, Overruns, StopRow, Stopped, Tally};
use file_ports::{Epoch, Files};
use grammar::Queue;
use newtype::Wrapping;
use ports::dispatch::{Clock, Dispatch, Moment, Shell, Wall};
use ports::store::Store;
use ports::transcript::{Body, SessionId};
use serde::{Deserialize, Serialize};

use super::super::Ground;
use super::super::crew::Crew;
use super::answers::{Answering, Answers, Search};
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
use harness::{Harness, Tasks};
use invariants::Ignoring;

#[derive(Debug, Serialize)]
pub struct Settings {
    hooks: BTreeMap<HookEvent, [Matcher; 1]>,
}

#[derive(Debug, Serialize)]
struct Matcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    matcher: Option<Tool>,
    hooks: [Handler; 1],
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Bash,
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
    queue: Option<Queue>,
}

#[derive(Clone, Copy, Debug)]
struct Queued<'queue>(&'queue Queue);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Continuing;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "bool")]
struct Continuation(Option<Continuing>);

#[derive(Debug, Serialize)]
pub struct Decision {
    decision: Verdict,
    reason: String,
}

#[derive(Debug, Serialize)]
pub enum Verdict {
    #[serde(rename = "block")]
    Block,
}

impl Settings {
    const KIND: &'static str = "command";

    pub fn listed(events: &[HookEvent]) -> Settings {
        Settings {
            hooks: events
                .iter()
                .map(|event| {
                    (*event, [Matcher { matcher: Settings::matcher(*event), hooks: [Settings::handler(*event)] }])
                })
                .collect(),
        }
    }

    const fn matcher(event: HookEvent) -> Option<Tool> {
        match event {
            HookEvent::Stop | HookEvent::SubagentStop => None,
            HookEvent::PostToolUse => Some(Tool::Bash),
        }
    }

    fn handler(event: HookEvent) -> Handler {
        Handler {
            kind: Settings::KIND,
            command: format!("design --workspace \"$CLAUDE_PROJECT_DIR\" hook {event}"),
            timeout: Settings::seconds(event),
        }
    }

    const fn seconds(event: HookEvent) -> u32 {
        match event {
            HookEvent::Stop => 3600,
            HookEvent::SubagentStop => 1500,
            HookEvent::PostToolUse => 300,
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

    pub fn decided<S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        crew: &Crew<'_, D, C, G>,
        observed: &Observed<'_, S>,
    ) -> Result<Uttered, Failure> {
        match self.continuation.0 {
            Some(Continuing) => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::Stop)))),
            None => Decision::told(&observed.blocking(crew, self)),
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
    pub fn new(store: &'store S, tasks: Option<Tasks>, started: Moment, queue: Option<Queue>) -> Observed<'store, S> {
        Observed { store, tasks, started, queue }
    }

    fn queued(&self) -> Option<Queued<'_>> {
        let idle = self.tasks.as_ref().map(Halt::from).filter(|halt| *halt == Halt::Stalled);
        idle.and(self.queue.as_ref()).filter(|queue| queue.items().is_empty().not()).map(Queued)
    }

    fn blocking<D: Dispatch, C: Clock, G: Shell>(&self, crew: &Crew<'_, D, C, G>, stopping: &Stopping) -> Vec<String> {
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
            .chain(self.queued().map(|queued| queued.to_string()))
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

impl Queued<'_> {
    const HEADER: &'static str = "Open items in transcripts/queue.md:";
}

impl fmt::Display for Queued<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Queued::HEADER)?;
        self.0.items().iter().try_for_each(|item| writeln!(formatter, "{item}"))
    }
}

impl From<bool> for Continuation {
    fn from(active: bool) -> Continuation {
        Continuation(active.then_some(Continuing))
    }
}

impl Decision {
    fn told(stopped: &[String]) -> Result<Uttered, Failure> {
        stopped.first().map_or(Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::Stop)))), |_first| {
            Decision::blocked(stopped.join("\n"))
        })
    }

    pub fn blocked(reason: String) -> Result<Uttered, Failure> {
        let json = serde_json::to_string(&Decision { decision: Verdict::Block, reason }).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Blocked)))
    }
}
