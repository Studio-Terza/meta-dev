mod harness;

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Not;
use std::path::PathBuf;

use console_ports::{Heard, Shown};
use design_messages::{LastSaid, Overran, Overruns, OwnerHistory, Said};
use dispatch_ports::{Clock, Shell, Wall};
use file_ports::{Epoch, Files};
use grammar::Lines;
use grammar_spec_line::Attachment;
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;
use transcript_ports::{Body, SessionId, Uuid};

use super::super::Ground;
use super::super::crew::Crew;
use super::answers::{Answering, Answers};
use super::subagent::Hub;
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Landed, Local, Named, Queue, Rooted};
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub(crate) use harness::Stopped;
use harness::{Halt, Harness, Tasks};
use invariants::Ignoring;

#[derive(Debug, Serialize)]
pub(crate) struct Settings {
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
pub(crate) struct Stopping {
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
pub(crate) struct Observed<'ground, 'run, F: Files, S: Store> {
    ground: &'ground Ground<'run, F, S>,
    tasks: Option<Tasks>,
}

#[derive(Debug)]
struct Queued<'shown, 'ground, 'run, F: Files, S: Store> {
    queue: &'shown Queue,
    landed: &'shown [Landed],
    naming: &'shown Naming<'ground, 'run, F, S>,
}

#[derive(Debug)]
struct Naming<'ground, 'run, F: Files, S: Store> {
    ground: &'ground Ground<'run, F, S>,
    specs: Option<&'ground Rooted>,
    owners: Vec<Said>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Continuing;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "bool")]
struct Continuation(Option<Continuing>);

#[derive(Debug, Serialize)]
pub(crate) struct Decision {
    decision: Verdict,
    reason: String,
}

#[derive(Debug, Serialize)]
pub(crate) enum Verdict {
    #[serde(rename = "block")]
    Block,
}

impl Settings {
    const KIND: &'static str = "command";

    pub(crate) fn listed(events: &[HookEvent]) -> Settings {
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
            HookEvent::PreToolUse | HookEvent::PostToolUse => Some(Tool::Bash),
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
            HookEvent::PreToolUse => 30,
            HookEvent::PostToolUse => 300,
        }
    }

    pub(crate) fn written(&self) -> Result<Uttered, Failure> {
        let json = serde_json::to_string_pretty(self).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Settings)))
    }
}

impl Stopping {
    pub(crate) fn heard(heard: Heard) -> Result<Stopping, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    const STALLED: &'static str =
        "Nothing is running: no background shell or agent will wake you, so the turn has stalled.";
    const OVERRUN: &'static str =
        "OVERRUN: these background tasks ran 20 minutes without finishing or writing output; check and stop them:";

    pub(crate) fn tasks<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>, now: Wall) -> Option<Tasks> {
        let harness = Harness::new(ground.files(), Epoch::new(now.into_inner()));
        self.transcript.as_deref().map(|transcript| harness.tasks(transcript))
    }

    pub(crate) fn decided<F: Files, S: Store, C: Clock, G: Shell>(
        &self,
        crew: &Crew<'_, C, G>,
        observed: &Observed<'_, '_, F, S>,
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

impl<'ground, 'run, F: Files, S: Store> Observed<'ground, 'run, F, S> {
    pub(crate) fn new(ground: &'ground Ground<'run, F, S>, tasks: Option<Tasks>) -> Observed<'ground, 'run, F, S> {
        Observed { ground, tasks }
    }

    fn blocking<C: Clock, G: Shell>(&self, crew: &Crew<'_, C, G>, stopping: &Stopping) -> Vec<String> {
        let store = self.ground.store();
        let last = stopping.said.clone().or_else(|| stopping.stored(store));
        let answers = last
            .as_ref()
            .and_then(|said| Answering::new(store, crew).searched(said).ignoring(Ignored::Unanswered))
            .flatten();
        let reported: Vec<Body> = store.read(Overruns).ignoring(Ignored::Unanswered).unwrap_or_default();
        let fresh: Vec<Body> =
            self.tasks.iter().flat_map(Tasks::hung).filter(|task| reported.contains(task).not()).cloned().collect();
        let halt =
            fresh.first().map_or_else(|| self.tasks.as_ref().map_or(Halt::Live, Halt::from), |_first| Halt::Overrun);
        let queued = (halt == Halt::Stalled).then(|| self.queued(crew)).flatten();
        store.write(Overran::from(fresh.clone())).ignoring(Ignored::Unanswered);
        Stopping::lead(halt, &fresh).into_iter().chain(answers.as_ref().map(Answers::to_string)).chain(queued).collect()
    }

    fn queued<C: Clock, G: Shell>(&self, crew: &Crew<'_, C, G>) -> Option<String> {
        let local = self.ground.local();
        let file = local.queue()?;
        match self.ground.queue(file) {
            Ok(queue) => self.drained(crew, &local, &queue),
            Err(unread) => Some(unread.to_string()),
        }
    }

    fn drained<C: Clock, G: Shell>(&self, crew: &Crew<'_, C, G>, local: &Local, queue: &Queue) -> Option<String> {
        queue.items().first()?;
        let landed = local
            .repository()
            .and_then(|repository| Hub::new(crew, crew.scratch(), repository).landed())
            .unwrap_or_default();
        let drained = queue.clone().drained(&landed, |path| self.ground.kind(path));
        let file = local.queue().filter(|_file| drained != *queue);
        file.map(|rewritten| self.ground.queued(rewritten, &drained).ignoring(Ignored::Unrewritten));
        let owners = self.ground.store().read(OwnerHistory).ignoring(Ignored::Unanswered).unwrap_or_default();
        let naming = Naming { ground: self.ground, specs: local.specs(), owners };
        drained.items().first().map(|_open| Queued { queue: &drained, landed: &landed, naming: &naming }.to_string())
    }
}

impl<F: Files, S: Store> Named for Naming<'_, '_, F, S> {
    fn spec(&self, cited: &Attachment) -> Option<String> {
        self.specs.and_then(|specs| self.ground.titled(specs, cited)).map(|heading| heading.to_string())
    }

    fn owner(&self, message: &str) -> Option<String> {
        let said = self.owners.iter().find(|said| {
            let uuid = AsRef::<Uuid>::as_ref(*said).to_string();
            uuid.get(..message.len()) == Some(message)
        })?;
        let body: &Body = said.as_ref();
        let lines = match body.as_ref().parse::<Lines>() {
            Ok(lines) => lines,
            Err(unsplit) => match unsplit {},
        };
        lines.into_iter().next()
    }
}

impl<F: Files, S: Store> Queued<'_, '_, '_, F, S> {
    const HEADER: &'static str = "Open items in the queue:";
}

impl<F: Files, S: Store> fmt::Display for Queued<'_, '_, '_, F, S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Self::HEADER)?;
        self.queue.items().iter().try_for_each(|item| writeln!(formatter, "{}", item.shown(self.landed, self.naming)))
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

    pub(crate) fn blocked(reason: String) -> Result<Uttered, Failure> {
        let json = serde_json::to_string(&Decision { decision: Verdict::Block, reason }).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Blocked)))
    }
}
