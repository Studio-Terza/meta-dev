mod harness;

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Not;
use std::path::{Path, PathBuf};

use console_ports::Shown;
use design_messages::{EndedTasks, LastSaid, Overran, Overruns, OwnerHistory, Said};
use dispatch_ports::{Ran, Timeout, Wall};
use file_ports::{Epoch, Files};
use grammar::lines::Lines;
use grammar::spec_line::Attachment;
use json_ports::Json;
use newtype::Wrapping;
use store_ports::Store;
use transcript_ports::{Body, SessionId, Uuid};

use super::super::Ground;
use super::super::crew::{Asked, Tape};
use super::super::ingest::Ingest;
use super::answers::{Answers, Moment};
use super::reply::{Hook, Reply};
use super::subagent::Hub;
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Landed, Local, Named, Queue, Rooted};
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub use harness::Stopped;
use harness::{Halt, Harness, Tasks};
use invariants::Ignoring;

#[derive(Debug, serde::Serialize)]
pub struct Settings {
    hooks: BTreeMap<HookEvent, Vec<Matcher>>,
}

#[derive(Debug, serde::Serialize)]
struct Matcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    matcher: Option<Tool>,
    hooks: [Handler; 1],
}

#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Bash,
    #[serde(rename = "Agent|SendMessage")]
    Briefs,
}

#[derive(Debug, serde::Serialize)]
struct Handler {
    #[serde(rename = "type")]
    kind: &'static str,
    command: String,
    timeout: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct Stopping {
    #[serde(rename = "stop_hook_active")]
    continuation: Continuation,
    #[serde(rename = "last_assistant_message", default)]
    said: Option<Body>,
    #[serde(rename = "session_id", default)]
    session: Option<SessionId>,
}

#[derive(Debug)]
pub struct Observed<'ground, 'run, F: Files, S: Store> {
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

#[derive(Debug)]
struct Stop {
    halt: Halt,
    fresh: Vec<Body>,
    answers: Option<Answers>,
    queue: Option<Drained>,
}

#[derive(Debug)]
enum Drained {
    Unread(String),
    Read { file: Rooted, queue: Queue, changed: Option<()>, shown: Option<String> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Continuing;

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "bool")]
struct Continuation(Option<Continuing>);

impl Settings {
    const KIND: &'static str = "command";
    const TEMPORARY: &'static str = "/private/tmp";
    const HARNESS: &'static str = "claude-";
    const IDENTITY: &'static str = "id";
    const QUICK: Timeout = Timeout::new(60);

    pub fn listed(events: &[HookEvent], harness: &Path) -> Settings {
        Settings {
            hooks: events
                .iter()
                .map(|event| {
                    let matchers = Settings::matchers(*event)
                        .iter()
                        .map(|matcher| Matcher {
                            matcher: *matcher,
                            hooks: [Settings::handler(*event, *matcher, harness)],
                        })
                        .collect();
                    (*event, matchers)
                })
                .collect(),
        }
    }

    pub fn harness<F: Files, S: Store>(ground: &Ground<'_, F, S>, tape: &Tape<'_>) -> Result<PathBuf, Failure> {
        ground.harness().map_or_else(
            || Settings::identified(ground, tape),
            |configured| Ok(ground.rendered(configured).to_path_buf()),
        )
    }

    fn identified<F: Files, S: Store>(ground: &Ground<'_, F, S>, tape: &Tape<'_>) -> Result<PathBuf, Failure> {
        let ran = tape.spoke(Self::IDENTITY, &["-u"], ground.workspace(), Self::QUICK)?;
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(Failure::Unidentified)?;
        let first = Lines::from(printed.as_ref()).into_iter().next();
        let user: u32 =
            first.and_then(|line| line.parse().ignoring(Ignored::Unidentified)).ok_or(Failure::Unidentified)?;
        Ok(Path::new(Self::TEMPORARY).join(format!("{}{user}", Self::HARNESS)))
    }

    const fn matchers(event: HookEvent) -> &'static [Option<Tool>] {
        match event {
            HookEvent::Stop | HookEvent::SubagentStop => &[None],
            HookEvent::PreToolUse => &[Some(Tool::Bash), Some(Tool::Briefs)],
            HookEvent::PostToolUse => &[Some(Tool::Bash)],
        }
    }

    fn handler(event: HookEvent, matcher: Option<Tool>, harness: &Path) -> Handler {
        Handler {
            kind: Settings::KIND,
            command: format!(
                "design --workspace \"$CLAUDE_PROJECT_DIR\" --harness \"{}\" hook {event}",
                harness.display()
            ),
            timeout: Settings::seconds(event, matcher),
        }
    }

    const fn seconds(event: HookEvent, matcher: Option<Tool>) -> u32 {
        match event {
            HookEvent::Stop => 3600,
            HookEvent::SubagentStop => 1500,
            HookEvent::PreToolUse => match matcher {
                Some(Tool::Briefs) => 300,
                Some(Tool::Bash) | None => 30,
            },
            HookEvent::PostToolUse => 1800,
        }
    }

    pub fn written(&self) -> Result<Uttered, Failure> {
        let json = Json::pretty(self).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Settings)))
    }
}

impl Hook for Stopping {
    fn answered<F: Files + Clone, S: Store>(
        &self,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure> {
        let store = ground.store();
        ground.prepared(tape.scratch())?;
        Ingest::read(store, &ground.transcripts())?;
        let stopped = Stopped::from(store.read(EndedTasks)?);
        let tasks = self.tasks(ground, tape.now()).map(|tasks| tasks.unstopped(&stopped));
        Ok(self.decided(tape, &Observed::new(ground, tasks))?)
    }
}

impl Stopping {
    const STALLED: &'static str =
        "Nothing is running: no background shell or agent will wake you, so the turn has stalled.";
    const OVERRUN: &'static str =
        "OVERRUN: these background tasks ran 20 minutes without finishing or writing output; check and stop them:";

    pub fn tasks<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>, now: Wall) -> Option<Tasks> {
        let harness = Harness::new(ground.files(), ground.harness(), Epoch::new(now.into_inner()));
        self.session.as_ref().map(|session| harness.tasks(&ground.transcript(session)))
    }

    fn decided<F: Files, S: Store>(
        &self,
        tape: &Tape<'_>,
        observed: &Observed<'_, '_, F, S>,
    ) -> Result<Reply<'static>, Asked> {
        let stopped = match self.continuation.0 {
            Some(Continuing) => Vec::new(),
            None => observed.acted(observed.measured(tape, self)?),
        };
        let passed = Reply::Pass(Outcome::Hooked(HookEvent::Stop));
        Ok(stopped.first().map_or(passed, |_first| Reply::Block { reason: stopped.join("\n") }))
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
    pub fn new(ground: &'ground Ground<'run, F, S>, tasks: Option<Tasks>) -> Observed<'ground, 'run, F, S> {
        Observed { ground, tasks }
    }

    fn measured(&self, tape: &Tape<'_>, stopping: &Stopping) -> Result<Stop, Asked> {
        let store = self.ground.store();
        let last = stopping.said.clone().or_else(|| stopping.stored(store));
        let owners = store.read(OwnerHistory).ignoring(Ignored::Unanswered);
        let asked = last.as_ref().zip(owners.as_ref());
        let answers = match asked {
            Some((said, history)) => Answers::searched(tape, said, Moment::Stopping, history)?,
            None => None,
        };
        let reported: Vec<Body> = store.read(Overruns).ignoring(Ignored::Unanswered).unwrap_or_default();
        let fresh: Vec<Body> =
            self.tasks.iter().flat_map(Tasks::hung).filter(|task| reported.contains(task).not()).cloned().collect();
        let halt =
            fresh.first().map_or_else(|| self.tasks.as_ref().map_or(Halt::Live, Halt::from), |_first| Halt::Overrun);
        let queue = match halt {
            Halt::Stalled => self.queued(tape, owners.unwrap_or_default())?,
            Halt::Overrun | Halt::Live => None,
        };
        Ok(Stop { halt, fresh, answers, queue })
    }

    fn acted(&self, stop: Stop) -> Vec<String> {
        let Stop { halt, fresh, answers, queue } = stop;
        self.ground.store().write(Overran::from(fresh.clone())).ignoring(Ignored::Unanswered);
        let shown = queue.and_then(|drained| match drained {
            Drained::Unread(said) => Some(said),
            Drained::Read { file, queue: kept, changed, shown } => {
                changed.map(|()| self.ground.queued(&file, &kept).ignoring(Ignored::Unrewritten));
                shown
            }
        });
        Stopping::lead(halt, &fresh).into_iter().chain(answers.as_ref().map(Answers::to_string)).chain(shown).collect()
    }

    fn queued(&self, tape: &Tape<'_>, owners: Vec<Said>) -> Result<Option<Drained>, Asked> {
        let local = self.ground.local();
        match local.queue().map(|file| (file, self.ground.queue(file))) {
            Some((file, Ok(queue))) if !queue.items().is_empty() => {
                self.drained(tape, &local, file, &queue, owners).map(Some)
            }
            Some((_, Ok(_))) | None => Ok(None),
            Some((_file, Err(unread))) => Ok(Some(Drained::Unread(unread.to_string()))),
        }
    }

    fn drained(
        &self,
        tape: &Tape<'_>,
        local: &Local,
        file: &Rooted,
        queue: &Queue,
        owners: Vec<Said>,
    ) -> Result<Drained, Asked> {
        let listed = match local.repository() {
            Some(repository) => Hub::new(tape.scratch(), repository).landed(tape)?,
            None => None,
        };
        let landed = listed.unwrap_or_default();
        let drained = queue.clone().drained(&landed, |path| self.ground.kind(path));
        let changed = (drained != *queue).then_some(());
        let naming = Naming { ground: self.ground, specs: local.specs(), owners };
        let shown = drained
            .items()
            .first()
            .map(|_open| Queued { queue: &drained, landed: &landed, naming: &naming }.to_string());
        Ok(Drained::Read { file: file.clone(), queue: drained, changed, shown })
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
        Lines::from(body.as_ref()).into_iter().next()
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
