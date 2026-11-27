use std::fmt;
use std::path::{Path, PathBuf};
use std::slice;

use dispatch_ports::{Call, Clock, Deadline, Pause, Ran, Shell, Timeout, Wall};
use invariants::Ignoring;
use serde::Deserialize;

use crate::cli::request::ignored::Ignored;

#[derive(Debug)]
pub(crate) struct Crew<'ports, C: Clock, G: Shell> {
    clock: &'ports C,
    shell: &'ports G,
    scratch: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Prompt(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Reply(String);

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

    pub(crate) fn new(clock: &'ports C, shell: &'ports G, scratch: PathBuf) -> Crew<'ports, C, G> {
        Crew { clock, shell, scratch }
    }

    pub(crate) fn now(&self) -> Wall {
        self.clock.now()
    }

    pub(crate) fn scratch(&self) -> &Path {
        &self.scratch
    }

    pub(crate) fn spoke(
        &self,
        program: &'static str,
        arguments: &[&str],
        directory: &Path,
        timeout: Timeout,
    ) -> Option<Ran> {
        let spoken = arguments.iter().map(|word| (*word).to_owned()).collect();
        let call = Call::new(program, spoken, directory.to_path_buf(), Deadline::after(self.clock.now(), timeout));
        self.shell.ran(&call).ignoring(Ignored::Unrun)
    }

    pub(crate) fn paused(&self, pause: Pause) {
        self.clock.paused(pause);
    }

    pub(crate) fn replied(&self, prompt: &Prompt, timeout: Timeout) -> Option<Reply> {
        self.fanned(slice::from_ref(prompt), timeout).into_iter().next().flatten()
    }

    pub(crate) fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Vec<Option<Reply>> {
        let deadline = Deadline::after(self.clock.now(), timeout);
        let asked: Vec<String> = Self::ASKED.map(str::to_owned).to_vec();
        let calls: Vec<Call> = prompts
            .iter()
            .map(|prompt| Call::new(Self::MODEL, asked.clone(), self.scratch.clone(), deadline).fed(prompt.0.clone()))
            .collect();
        self.shell
            .fanned(&calls)
            .into_iter()
            .map(|ran| ran.ignoring(Ignored::Unrun).as_ref().and_then(Ran::succeeded).and_then(Reply::read))
            .collect()
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
