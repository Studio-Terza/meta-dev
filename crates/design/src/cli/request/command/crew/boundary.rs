use std::fmt;
use std::path::PathBuf;

use dispatch_ports::{Call, Deadline, Printed};
use invariants::Ignoring;
use serde::Deserialize;

use super::Program;
use crate::cli::request::ignored::Ignored;

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

impl Program {
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

    pub(crate) fn called(
        self,
        arguments: impl IntoIterator<Item: fmt::Display>,
        directory: PathBuf,
        deadline: Deadline,
    ) -> Call {
        let spoken = arguments.into_iter().map(|word| word.to_string()).collect();
        Call::new(self.into(), spoken, directory, deadline)
    }
}

impl Prompt {
    pub(crate) fn asked(&self, directory: PathBuf, deadline: Deadline) -> Call {
        Program::Claude.called(Program::ASKED, directory, deadline).fed(self.0.clone())
    }
}

impl<Shown: fmt::Display> From<&Shown> for Prompt {
    fn from(shown: &Shown) -> Prompt {
        Prompt(shown.to_string())
    }
}

impl Reply {
    pub(crate) fn read(printed: &Printed) -> Option<Reply> {
        let envelope: Envelope = serde_json::from_str(printed.as_ref()).ignoring(Ignored::Unreplied)?;
        (envelope.subtype == Subtype::Success).then_some(envelope.result).flatten().map(Reply)
    }
}

impl AsRef<str> for Reply {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
