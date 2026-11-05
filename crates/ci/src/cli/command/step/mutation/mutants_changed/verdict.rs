use std::fmt;
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::command::step::mutation::cargo_mutants::CargoMutants;
use crate::cli::ports::{Exit, Text};

#[derive(Clone, Copy, Debug)]
pub(super) enum Verdict<'text> {
    Skipped,
    NoneMissed,
    Missed(&'text Text),
}

impl<'text> From<&'text Text> for Verdict<'text> {
    fn from(missed: &'text Text) -> Verdict<'text> {
        missed.as_ref().chars().next().map_or(Verdict::NoneMissed, |_| Verdict::Missed(missed))
    }
}

impl Verdict<'_> {
    pub(super) fn reported(self, out: &mut impl Write) -> Result<Exit, Failure<'static>> {
        write!(out, "{self}")?;
        Ok(self.exit())
    }

    fn exit(self) -> Exit {
        match self {
            Verdict::Skipped | Verdict::NoneMissed => Exit::Succeeded,
            Verdict::Missed(_) => Exit::Failed(CargoMutants::MISSED),
        }
    }
}

impl fmt::Display for Verdict<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Skipped => {
                writeln!(formatter, "mutants: skipped, no crate is reached by this change, so nothing was tested")
            }
            Verdict::NoneMissed => writeln!(formatter, "mutants: none missed"),
            Verdict::Missed(missed) => write!(formatter, "mutants missed:\n{missed}"),
        }
    }
}
