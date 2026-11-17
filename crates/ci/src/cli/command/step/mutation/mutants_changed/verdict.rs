use std::fmt;

use crate::cli::command::Told;
use crate::cli::command::step::mutation::cargo_mutants::CargoMutants;
use crate::cli::ports::{Exit, Text};

#[derive(Clone, Copy, Debug)]
pub(super) enum Verdict<'text> {
    Skipped,
    Unchanged,
    NoneMissed,
    Missed(&'text Text),
}

impl<'text> From<&'text Text> for Verdict<'text> {
    fn from(missed: &'text Text) -> Verdict<'text> {
        missed.as_ref().chars().next().map_or(Verdict::NoneMissed, |_| Verdict::Missed(missed))
    }
}

impl Verdict<'_> {
    pub(super) fn told<'step>(self) -> Vec<Told<'step>> {
        vec![Told::said(self), Told::Ended(Ok(self.exit()))]
    }

    fn exit(self) -> Exit {
        match self {
            Verdict::Skipped | Verdict::Unchanged | Verdict::NoneMissed => Exit::Success,
            Verdict::Missed(_) => Exit::Failure(CargoMutants::MISSED),
        }
    }
}

impl fmt::Display for Verdict<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Skipped => {
                writeln!(formatter, "mutants: skipped, no crate is reached by this change, so nothing was tested")
            }
            Verdict::Unchanged => {
                writeln!(formatter, "mutants: none selected, no changed line since the prior run and no survivor of it")
            }
            Verdict::NoneMissed => writeln!(formatter, "mutants: none missed"),
            Verdict::Missed(missed) => write!(formatter, "mutants missed:\n{missed}"),
        }
    }
}
