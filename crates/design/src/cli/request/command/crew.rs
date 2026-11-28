use std::cell::Cell;
use std::fmt;
use std::path::{Path, PathBuf};
use std::slice;

use dispatch_ports::{Call, Clock, Deadline, Pause, Ran, Shell, Timeout, Wall};
use invariants::Ignoring;
use serde::Deserialize;

use crate::cli::request::ignored::Ignored;

#[derive(Debug)]
pub struct Crew<'ports, C: Clock, G: Shell> {
    clock: &'ports C,
    shell: &'ports G,
    scratch: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Ran(Vec<Call>),
    Paused(Pause),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    at: Wall,
    ran: Vec<Option<Ran>>,
}

#[derive(Debug)]
pub struct Tape<'heard> {
    start: Wall,
    scratch: &'heard Path,
    heard: &'heard [Heard],
    read: Cell<usize>,
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

#[derive(Debug, Deserialize)]
struct Envelope {
    subtype: Subtype,
    result: Option<String>,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Subtype {
    #[serde(rename = "success")]
    Success,
    #[serde(other)]
    Other,
}

impl<'ports, C: Clock, G: Shell> Crew<'ports, C, G> {
    pub fn new(clock: &'ports C, shell: &'ports G, scratch: PathBuf) -> Crew<'ports, C, G> {
        Crew { clock, shell, scratch }
    }

    pub fn now(&self) -> Wall {
        self.clock.now()
    }

    pub fn scratch(&self) -> &Path {
        &self.scratch
    }

    pub fn driven<T>(&self, machine: impl Fn(&Tape<'_>) -> Result<T, Asked>) -> T {
        self.replayed(&machine, self.clock.now(), Vec::new())
    }

    fn replayed<T>(&self, machine: &impl Fn(&Tape<'_>) -> Result<T, Asked>, start: Wall, heard: Vec<Heard>) -> T {
        match machine(&Tape { start, scratch: &self.scratch, heard: &heard, read: Cell::default() }) {
            Ok(done) => done,
            Err(asked) => self.replayed(machine, start, [heard, vec![self.performed(asked)]].concat()),
        }
    }

    fn performed(&self, asked: Asked) -> Heard {
        match asked {
            Asked::Ran(calls) => {
                let ran = self.shell.fanned(&calls).into_iter().map(|ran| ran.ignoring(Ignored::Unrun)).collect();
                Heard { at: self.clock.now(), ran }
            }
            Asked::Paused(pause) => Heard { at: self.clock.paused(pause), ran: Vec::new() },
        }
    }
}

impl Tape<'_> {
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

    pub fn now(&self) -> Wall {
        let last = self.read.get().checked_sub(1).and_then(|last| self.heard.get(last));
        last.map_or(self.start, |heard| heard.at)
    }

    pub fn scratch(&self) -> &Path {
        self.scratch
    }

    fn heard(&self, asked: Asked) -> Result<&Heard, Asked> {
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
        self.heard(Asked::Paused(pause)).map(|heard| heard.at)
    }

    pub fn replied(&self, prompt: &Prompt, timeout: Timeout) -> Result<Option<Reply>, Asked> {
        Ok(self.fanned(slice::from_ref(prompt), timeout)?.into_iter().next().flatten())
    }

    pub fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Result<Vec<Option<Reply>>, Asked> {
        let deadline = Deadline::after(self.now(), timeout);
        let asked: Vec<String> = Self::ASKED.map(str::to_owned).to_vec();
        let calls: Vec<Call> = prompts
            .iter()
            .map(|prompt| {
                Call::new(Self::MODEL, asked.clone(), self.scratch.to_path_buf(), deadline).fed(prompt.0.clone())
            })
            .collect();
        let heard = self.heard(Asked::Ran(calls))?;
        Ok(heard.ran.iter().map(|ran| ran.as_ref().and_then(Ran::succeeded).and_then(Reply::read)).collect())
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

impl<Shown: fmt::Display> From<&Shown> for Prompt {
    fn from(shown: &Shown) -> Prompt {
        Prompt(shown.to_string())
    }
}

impl Reply {
    fn read(printed: &dispatch_ports::Printed) -> Option<Reply> {
        let envelope: Envelope = serde_json::from_str(printed.as_ref()).ignoring(Ignored::Unreplied)?;
        (envelope.subtype == Subtype::Success).then_some(envelope.result).flatten().map(Reply)
    }
}

impl AsRef<str> for Reply {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
