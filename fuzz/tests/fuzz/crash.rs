use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::input::Input;

#[derive(Debug)]
pub enum Kept {
    At(PathBuf),
    Lost,
}

impl From<Option<PathBuf>> for Kept {
    fn from(kept: Option<PathBuf>) -> Kept {
        match kept {
            Some(path) => Kept::At(path),
            None => Kept::Lost,
        }
    }
}

impl fmt::Display for Kept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kept::At(path) => write!(formatter, "input kept at {}", path.display()),
            Kept::Lost => formatter.write_str("input not kept"),
        }
    }
}

#[derive(Debug)]
pub struct Failing {
    input: Input,
    reason: String,
    kept: Kept,
}

impl Failing {
    pub fn new(input: Input, reason: String, kept: Kept) -> Failing {
        Failing { input, reason, kept }
    }

    pub fn input(&self) -> &Input {
        &self.input
    }
}

impl fmt::Display for Failing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a {} input b\"{}\": {}; {}",
            self.input.name(),
            self.input.bytes().escape_ascii(),
            self.reason,
            self.kept
        )
    }
}

#[derive(Debug)]
pub enum Crash {
    Panicked(Failing),
    Failed(Failing),
    Unrepeated(Failing),
    Aborted { reason: String },
}

impl fmt::Display for Crash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Crash::Panicked(failing) => write!(formatter, "the runner panicked on {failing}"),
            Crash::Failed(failing) => write!(formatter, "two runs of the runner disagreed on {failing}"),
            Crash::Unrepeated(failing) => {
                write!(formatter, "the runner failed once but not when replayed on {failing}")
            }
            Crash::Aborted { reason } => write!(formatter, "the campaign stopped: {reason}"),
        }
    }
}

impl Error for Crash {}
