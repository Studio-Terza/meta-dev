use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::input::Input;

#[derive(Debug)]
pub enum Crash {
    Panicked { input: Input, reason: String, kept: Option<PathBuf> },
    Aborted { reason: String },
}

impl fmt::Display for Crash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Crash::Panicked { input, reason, kept: Some(kept) } => write!(
                formatter,
                "the runner panicked on a {} input: {reason}; input kept at {}",
                input.name(),
                kept.display()
            ),
            Crash::Panicked { input, reason, kept: None } => {
                write!(formatter, "the runner panicked on a {} input: {reason}; input not kept", input.name())
            }
            Crash::Aborted { reason } => write!(formatter, "the campaign stopped: {reason}"),
        }
    }
}

impl Error for Crash {}
