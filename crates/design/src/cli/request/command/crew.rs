use std::cell::{Cell, RefCell};
use std::fmt;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::Duration;

use dispatch_ports::{Call, Printed, Ran, Shell};
use invariants::Ignoring;
use json_ports::Json;
use pipeline::Measure;
use timer_ports::{Clock, Deadline, Elapsed, Pause, Timeout, Wall};

use crate::cli::request::ignored::Ignored;

#[derive(Debug)]
pub struct Crew<'ports, C: Clock, G: Shell> {
    clock: &'ports C,
    shell: &'ports G,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Ran(Vec<Call>),
    Paused(Pause),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(from = "Recorded")]
pub struct Answer {
    took: Elapsed,
    ran: Vec<Option<Ran>>,
}

#[derive(Debug, serde::Deserialize)]
enum Recorded {
    Succeeded(String),
    Failed(String),
    Overdue,
    Unrun,
    Paused(u64),
}

#[derive(Debug)]
pub struct Tape<'heard> {
    start: Wall,
    scratch: PathBuf,
    heard: &'heard [Answer],
    read: Cell<usize>,
    asked: RefCell<Vec<Asked>>,
}

#[derive(Debug)]
pub enum Cut<E> {
    Asked(Asked),
    Failed(E),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply(String);

#[derive(Debug, serde::Deserialize)]
struct Envelope {
    subtype: Subtype,
    result: Option<String>,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Subtype {
    #[serde(rename = "success")]
    Success,
    #[serde(other)]
    Other,
}

impl<'ports, C: Clock, G: Shell> Crew<'ports, C, G> {
    pub fn new(clock: &'ports C, shell: &'ports G) -> Crew<'ports, C, G> {
        Crew { clock, shell }
    }

    pub fn now(&self) -> Wall {
        self.clock.now()
    }

    pub fn performed(&self, start: Wall, heard: &[Answer], asked: &Asked) -> Answer {
        let before = Tape::elapsed(start, heard);
        let (after, ran) = match asked {
            Asked::Ran(calls) => (
                self.clock.now(),
                self.shell.fanned(calls).into_iter().map(|ran| ran.ignoring(Ignored::Unrun)).collect(),
            ),
            Asked::Paused(pause) => (self.clock.paused(*pause), Vec::new()),
        };
        Answer { took: after.since(before), ran }
    }
}

impl<'heard> Tape<'heard> {
    const MODEL: &'static str = "claude";
    const ASKED: [&'static str; 10] = [
        "-p",
        "--output-format",
        "json",
        "--setting-sources",
        "user",
        "--strict-mcp-config",
        "--tools",
        "",
        "--model",
        "haiku",
    ];

    pub fn new(start: Wall, scratch: PathBuf, heard: &'heard [Answer]) -> Tape<'heard> {
        Tape { start, scratch, heard, read: Cell::default(), asked: RefCell::default() }
    }

    fn elapsed(start: Wall, heard: &[Answer]) -> Wall {
        start.after(Elapsed::total(heard.iter().map(|answer| answer.took)))
    }

    pub fn now(&self) -> Wall {
        Tape::elapsed(self.start, self.heard.get(..self.read.get()).unwrap_or_default())
    }

    pub fn scratch(&self) -> &Path {
        &self.scratch
    }

    pub fn asked(self) -> Vec<Asked> {
        self.asked.into_inner()
    }

    fn heard(&self, asked: Asked) -> Result<&Answer, Asked> {
        self.asked.borrow_mut().push(asked.clone());
        let at = self.read.get();
        let heard = self.heard.get(at).ok_or(asked)?;
        self.read.set(at + 1);
        Ok(heard)
    }

    pub fn spoke(
        &self,
        program: &'static str,
        arguments: &[&str],
        directory: &Path,
        timeout: Timeout,
    ) -> Result<Option<Ran>, Asked> {
        let spoken = arguments.iter().map(|word| (*word).to_owned()).collect();
        let call = Call::new(program, spoken, directory.to_path_buf(), Deadline::after(self.now(), timeout));
        Ok(self.heard(Asked::Ran(vec![call]))?.ran.first().cloned().flatten())
    }

    pub fn paused(&self, pause: Pause) -> Result<Wall, Asked> {
        self.heard(Asked::Paused(pause))?;
        Ok(self.now())
    }

    pub fn replied(&self, prompt: &Prompt, timeout: Timeout) -> Result<Option<Reply>, Asked> {
        Ok(self.fanned(slice::from_ref(prompt), timeout)?.into_iter().next().flatten())
    }

    pub fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Result<Vec<Option<Reply>>, Asked> {
        let deadline = Deadline::after(self.now(), timeout);
        let asked: Vec<String> = Self::ASKED.map(str::to_owned).to_vec();
        let calls: Vec<Call> = prompts
            .iter()
            .map(|prompt| Call::new(Self::MODEL, asked.clone(), self.scratch.clone(), deadline).fed(prompt.0.clone()))
            .collect();
        let ran = match calls.first() {
            Some(_first) => self.heard(Asked::Ran(calls))?.ran.clone(),
            None => Vec::new(),
        };
        Ok(ran.iter().map(|call| call.as_ref().and_then(Ran::succeeded).and_then(Reply::read)).collect())
    }
}

impl<E> Cut<E> {
    pub fn settled<T>(cut: Result<T, Cut<E>>) -> Result<Result<T, E>, Asked> {
        match cut {
            Ok(done) => Ok(Ok(done)),
            Err(Cut::Failed(failed)) => Ok(Err(failed)),
            Err(Cut::Asked(asked)) => Err(asked),
        }
    }
}

impl<E> From<Asked> for Cut<E> {
    fn from(asked: Asked) -> Cut<E> {
        Cut::Asked(asked)
    }
}

impl From<Recorded> for Answer {
    fn from(recorded: Recorded) -> Answer {
        let ran = |ran: Option<Ran>| Answer { took: Elapsed::default(), ran: vec![ran] };
        match recorded {
            Recorded::Succeeded(printed) => ran(Some(Ran::Succeeded(Printed::new(printed)))),
            Recorded::Failed(printed) => ran(Some(Ran::Failed(Printed::new(printed)))),
            Recorded::Overdue => ran(Some(Ran::Overdue)),
            Recorded::Unrun => ran(None),
            Recorded::Paused(seconds) => Answer { took: Elapsed::from(Duration::from_secs(seconds)), ran: Vec::new() },
        }
    }
}

impl<Shown: fmt::Display> From<&Shown> for Prompt {
    fn from(shown: &Shown) -> Prompt {
        Prompt(shown.to_string())
    }
}

impl Reply {
    fn read(printed: &Printed) -> Option<Reply> {
        let envelope: Envelope = Json::read(printed.as_ref()).ignoring(Ignored::Unreplied)?;
        (envelope.subtype == Subtype::Success).then_some(envelope.result).flatten().map(Reply)
    }
}

impl AsRef<str> for Reply {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
