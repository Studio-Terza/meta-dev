use std::fmt;
use std::path::PathBuf;

use dispatch_ports::{Call, Printed, Ran, Wall};
use invariants::Unpoisoned;
use newtype::Wrapping;
use serde::Deserialize;

use super::Answering;
use crate::Count;

#[derive(Clone, Debug, Deserialize)]
pub struct Scripted {
    #[serde(default)]
    tool: Tool,
    asked: String,
    ran: Answered,
    #[serde(default)]
    printed: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, strum::IntoStaticStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum Tool {
    #[default]
    Gh,
    Git,
    Cargo,
    Design,
    Claude,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum Answered {
    Succeeded,
    Failed,
    Overdue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Asked {
    after: i64,
    program: &'static str,
    arguments: Vec<String>,
    directory: PathBuf,
    fed: Option<String>,
}

impl Scripted {
    const SPACE: &'static str = " ";

    pub(crate) fn answers(&self, call: &Call) -> Option<()> {
        let program: &'static str = self.tool.into();
        let asked = self.asked.as_str();
        let spoken = call.arguments().join(Scripted::SPACE) == asked;
        let fed = call.input().is_some_and(|fed| fed.contains(asked));
        (call.program() == program && (spoken || fed)).then_some(())
    }

    pub(crate) fn ran(&self) -> Ran {
        let printed = Printed::new(self.printed.clone());
        match self.ran {
            Answered::Succeeded => Ran::Succeeded(printed),
            Answered::Failed => Ran::Failed(printed),
            Answered::Overdue => Ran::Overdue,
        }
    }
}

impl Asked {
    const SHOWN: usize = 2;

    pub(crate) fn of(call: &Call, start: Wall) -> Asked {
        Asked {
            after: call.deadline().into_inner().into_inner().saturating_sub(start.into_inner()),
            program: call.program(),
            arguments: call.arguments().to_vec(),
            directory: call.directory().to_path_buf(),
            fed: call.input().map(str::to_owned),
        }
    }

    pub(crate) fn repeats<'log>(&self, before: impl Iterator<Item = &'log Asked>) -> usize {
        before
            .filter(|earlier| {
                earlier.program == self.program && earlier.arguments == self.arguments && earlier.fed == self.fed
            })
            .count()
    }
}

impl fmt::Debug for Answering {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = self.asked.lock().unpoisoned();
        let shown: Vec<&Asked> = log
            .iter()
            .enumerate()
            .filter(|(at, asked)| asked.repeats(log.iter().take(*at)) < Asked::SHOWN)
            .map(|(_at, asked)| asked)
            .collect();
        let hidden = log.len().saturating_sub(shown.len());
        formatter
            .debug_struct("Answering")
            .field("asked", &shown)
            .field("repeated", &Count::new(hidden))
            .field("last", &log.last().filter(|_last| hidden > 0))
            .finish_non_exhaustive()
    }
}
