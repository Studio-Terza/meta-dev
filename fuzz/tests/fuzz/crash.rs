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
    fn from(kept: Option<PathBuf>) -> Self {
        match kept {
            Some(path) => Self::At(path),
            None => Self::Lost,
        }
    }
}

impl fmt::Display for Kept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::At(path) => write!(formatter, "input kept at {}", path.display()),
            Self::Lost => formatter.write_str("input not kept"),
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
    pub fn new(input: Input, reason: String, kept: Kept) -> Self {
        Self { input, reason, kept }
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
            Self::Panicked(failing) => write!(formatter, "the runner panicked on {failing}"),
            Self::Failed(failing) => write!(formatter, "the runner failed a check on {failing}"),
            Self::Unrepeated(failing) => {
                write!(formatter, "the runner failed once but not when replayed on {failing}")
            }
            Self::Aborted { reason } => write!(formatter, "the campaign stopped: {reason}"),
        }
    }
}

impl Error for Crash {}
