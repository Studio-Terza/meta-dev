use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use design_model::Halt;
use design_store::Tally;
use file_ports::{Bytes, Epoch, Offset, Size, Stamp, Stamps, Tail};
use grammar_lines::Lines;
use invariants::Ignoring;
use newtype::Wrapping;
use serde::Deserialize;
use transcript_ports::Body;

use crate::cli::request::ignored::Ignored;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Runner {
    Shell,
    Agent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Finished,
    Live,
    Attended,
    Hung,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ended;

#[derive(Debug)]
pub struct Output {
    runner: Runner,
    path: PathBuf,
    name: Body,
}

#[derive(Debug)]
pub struct Task {
    runner: Runner,
    status: Status,
    name: Body,
}

#[derive(Debug, Default)]
pub struct Tasks {
    shells: usize,
    agents: usize,
    hung: Vec<Body>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Turn {
    Assistant,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
enum Reason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(other)]
    Other,
}

#[derive(Debug, Default, Deserialize)]
struct Closing {
    #[serde(default)]
    stop_reason: Option<Reason>,
}

#[derive(Debug, Deserialize)]
struct Last {
    #[serde(rename = "type", default)]
    turn: Turn,
    #[serde(default)]
    message: Closing,
}

impl Runner {
    const OUTPUT: &'static str = "output";
    const SHELL: u8 = b'b';
    const AGENT: u8 = b'a';

    fn spawned(path: &Path) -> Option<Runner> {
        let named = path.file_name().map(OsStr::as_encoded_bytes).and_then(<[u8]>::first).copied();
        let runner = named.and_then(|first| {
            (first == Self::SHELL).then_some(Runner::Shell).or_else(|| (first == Self::AGENT).then_some(Runner::Agent))
        });
        runner.filter(|_| path.extension() == Some(OsStr::new(Self::OUTPUT)))
    }

    const fn tail(self) -> u64 {
        match self {
            Runner::Shell => 1 << 8,
            Runner::Agent => 1 << 20,
        }
    }

    fn ending(self, text: &str) -> Option<Ended> {
        match self {
            Runner::Shell => text.contains(Task::TRAILER).then_some(Ended),
            Runner::Agent => Last::parsed(text).and_then(Last::ending),
        }
    }

    fn status(self, ending: Option<Ended>, stamps: Stamps, now: Epoch) -> Status {
        let fresh = |stamp: Stamp| now.into_inner() - stamps.dated(stamp).into_inner() < Task::QUIET;
        let open = || {
            fresh(Stamp::from(self))
                .then_some(Status::Live)
                .or_else(|| fresh(Stamp::Touched).then_some(Status::Attended))
                .unwrap_or(Status::Hung)
        };
        ending.map_or_else(open, |Ended| Status::Finished)
    }
}

impl Output {
    pub fn listed(path: &Path) -> Option<Output> {
        let runner = Runner::spawned(path)?;
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .and_then(|stem| Body::try_new(stem.to_owned()).ignoring(Ignored::Untasked))?;
        Some(Output { runner, path: path.to_path_buf(), name })
    }

    pub fn tailed(&self, size: Size) -> Option<Tail> {
        let from = i64::try_from(size.into_inner().saturating_sub(self.runner.tail())).ignoring(Ignored::Untasked)?;
        Some(Tail::new(self.path.clone(), Offset::try_new(from).ignoring(Ignored::Untasked)?))
    }
}

impl AsRef<Path> for Output {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl From<Runner> for Stamp {
    fn from(runner: Runner) -> Stamp {
        match runner {
            Runner::Shell => Stamp::Born,
            Runner::Agent => Stamp::Touched,
        }
    }
}

impl Last {
    fn parsed(text: &str) -> Option<Last> {
        let last = text.parse::<Lines>().ignoring(Ignored::Untasked)?.into_iter().last()?;
        serde_json::from_str::<Last>(&last).ignoring(Ignored::Untasked)
    }

    fn ending(self) -> Option<Ended> {
        match self.turn {
            Turn::Assistant => self.message.ending(),
            Turn::Other => None,
        }
    }
}

impl Closing {
    fn ending(&self) -> Option<Ended> {
        match self.stop_reason {
            Some(Reason::EndTurn) => Some(Ended),
            Some(Reason::Other) | None => None,
        }
    }
}

impl Task {
    const TRAILER: &'static str = "[exited with code";
    const QUIET: i64 = 20 * 60;
    const CONTINUATION: u8 = 0b1100_0000;
    const FOLLOWER: u8 = 0b1000_0000;

    pub fn new(Output { runner, name, .. }: Output, tail: &Bytes, stamps: Stamps, now: Epoch) -> Option<Task> {
        let whole: Vec<u8> =
            tail.as_ref().iter().copied().skip_while(|byte| byte & Self::CONTINUATION == Self::FOLLOWER).collect();
        let text = String::from_utf8(whole).ignoring(Ignored::Untasked)?;
        Some(Task { runner, status: runner.status(runner.ending(&text), stamps, now), name })
    }
}

impl From<&Tasks> for Halt {
    fn from(tasks: &Tasks) -> Halt {
        NonZeroUsize::new(tasks.running()).map_or(Halt::Stalled, |_running| Halt::Live)
    }
}

impl FromIterator<Task> for Tasks {
    fn from_iter<Read: IntoIterator<Item = Task>>(tasks: Read) -> Tasks {
        tasks.into_iter().fold(Tasks::default(), Tasks::counted)
    }
}

impl Tasks {
    fn counted(self, task: Task) -> Tasks {
        match task.status {
            Status::Live => self.running_by(task.runner),
            Status::Hung => self.hung_on(task.name),
            Status::Finished | Status::Attended => self,
        }
    }

    fn running_by(self, runner: Runner) -> Tasks {
        match runner {
            Runner::Shell => Tasks { shells: self.shells + 1, ..self },
            Runner::Agent => Tasks { agents: self.agents + 1, ..self },
        }
    }

    fn hung_on(self, name: Body) -> Tasks {
        Tasks { hung: self.hung.into_iter().chain([name]).collect(), ..self }
    }

    #[must_use]
    pub fn live(&self) -> [Tally; 2] {
        [Tally::from(self.shells), Tally::from(self.agents)]
    }

    #[must_use]
    pub fn running(&self) -> usize {
        self.shells + self.agents
    }

    #[must_use]
    pub fn hung(&self) -> &[Body] {
        &self.hung
    }
}
