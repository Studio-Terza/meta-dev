use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Epoch, Offset, Size, Stamp, Stamps, Tail};
use grammar::Grammar;
use grammar::notice::TaskFile;
use invariants::Ignoring;
use newtype::Wrapping;
use pipeline::Measure;
use transcript_ports::{Body, TaskId, Turn};

use crate::cli::request::ignored::Ignored;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    Live,
    Stalled,
    Overrun,
}

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
    Gone,
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
pub struct Stopped(BTreeSet<TaskId>);

pipeline::measured! {
    #[derive(Debug)]
    pub struct Tasks {
        shells: usize,
        agents: usize,
        hung: BTreeSet<Body>,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Written {
    Output,
    Transcript,
}

impl Written {
    const OUTPUT: &'static str = "output";
    const TRANSCRIPT: &'static str = "jsonl";

    fn read(path: &Path) -> Option<Written> {
        let extension = path.extension()?;
        (extension == Self::OUTPUT)
            .then_some(Written::Output)
            .or_else(|| (extension == Self::TRANSCRIPT).then_some(Written::Transcript))
    }

    fn kept_by(self, runner: Runner) -> Option<Runner> {
        match self {
            Written::Output => Some(runner),
            Written::Transcript => (runner == Runner::Agent).then_some(runner),
        }
    }
}

impl Runner {
    const SHELL: u8 = b'b';
    const AGENT: u8 = b'a';

    fn spawned(path: &Path) -> Option<Runner> {
        let named = path.file_name().map(OsStr::as_encoded_bytes).and_then(<[u8]>::first).copied();
        let runner = named.and_then(|first| {
            (first == Self::SHELL).then_some(Runner::Shell).or_else(|| (first == Self::AGENT).then_some(Runner::Agent))
        })?;
        Written::read(path)?.kept_by(runner)
    }

    const fn tail(self) -> u64 {
        match self {
            Runner::Shell => 1 << 8,
            Runner::Agent => 1 << 20,
        }
    }

    fn ending(self, text: &str) -> Option<Ended> {
        match self {
            Runner::Shell => Task::TRAILERS.iter().any(|trailer| text.contains(trailer)).then_some(Ended),
            Runner::Agent => (Turn::from(text) == Turn::Ended).then_some(Ended),
        }
    }

    fn status(self, ending: Option<Ended>, stamps: Stamps, now: Epoch) -> Status {
        let within = |stamp: Stamp, limit: i64| now.into_inner() - stamps.dated(stamp).into_inner() < limit;
        let open = || {
            within(Stamp::from(self), Task::QUIET)
                .then_some(Status::Live)
                .or_else(|| within(Stamp::Touched, Task::QUIET).then_some(Status::Attended))
                .or_else(|| within(Stamp::Touched, Task::GONE).then_some(Status::Hung))
                .unwrap_or(Status::Gone)
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

impl Task {
    const TRAILERS: [&'static str; 2] = ["[exited with code", "[killed]"];
    const QUIET: i64 = 20 * 60;
    const GONE: i64 = 60 * 60;
    const CONTINUATION: u8 = 0b1100_0000;
    const FOLLOWER: u8 = 0b1000_0000;

    pub fn new(Output { runner, name, .. }: Output, tail: &Bytes, stamps: Stamps, now: Epoch) -> Option<Task> {
        let whole: Vec<u8> =
            tail.as_ref().iter().copied().skip_while(|byte| byte & Self::CONTINUATION == Self::FOLLOWER).collect();
        let text = String::from_utf8(whole).ignoring(Ignored::Untasked)?;
        Some(Task { runner, status: runner.status(runner.ending(&text), stamps, now), name })
    }
}

impl Stopped {
    fn holding(&self, name: &Body) -> Option<Ended> {
        let file = TaskFile::read(name.as_ref())?;
        file.ids()
            .into_iter()
            .flatten()
            .find_map(|id| self.0.iter().find(|task| AsRef::<str>::as_ref(*task) == id))
            .map(|_stopped| Ended)
    }
}

impl From<Vec<TaskId>> for Stopped {
    fn from(tasks: Vec<TaskId>) -> Stopped {
        Stopped(tasks.into_iter().collect())
    }
}

impl From<&Tasks> for Halt {
    fn from(tasks: &Tasks) -> Halt {
        NonZeroUsize::new(tasks.running()).map_or(Halt::Stalled, |_running| Halt::Live)
    }
}

impl FromIterator<Task> for Tasks {
    fn from_iter<Read: IntoIterator<Item = Task>>(tasks: Read) -> Tasks {
        Tasks::total(tasks.into_iter().map(Tasks::from))
    }
}

impl From<Task> for Tasks {
    fn from(task: Task) -> Tasks {
        match (task.status, task.runner) {
            (Status::Live, Runner::Shell) => Tasks { shells: 1, ..Tasks::default() },
            (Status::Live, Runner::Agent) => Tasks { agents: 1, ..Tasks::default() },
            (Status::Hung, _) => Tasks { hung: BTreeSet::from([task.name]), ..Tasks::default() },
            (Status::Finished | Status::Attended | Status::Gone, _) => Tasks::default(),
        }
    }
}

impl Tasks {
    #[must_use]
    pub fn unstopped(self, stopped: &Stopped) -> Tasks {
        Tasks { hung: self.hung.into_iter().filter(|name| stopped.holding(name).is_none()).collect(), ..self }
    }

    #[must_use]
    pub fn running(&self) -> usize {
        self.shells + self.agents
    }

    #[must_use]
    pub fn hung(&self) -> &BTreeSet<Body> {
        &self.hung
    }
}
