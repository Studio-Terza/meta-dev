use std::fmt;
use std::io::Write;

use super::change::Change;
use super::check::Check;
use crate::cli::command::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Beyond(Check),
    Refactor,
}

#[derive(Debug)]
pub struct Owed<'change> {
    fault: Fault,
    failing: Vec<&'change Change>,
}

impl<'change> Owed<'change> {
    pub(crate) fn new(fault: Fault, failing: Vec<&'change Change>) -> Self {
        Owed { fault, failing }
    }

    pub(crate) fn outstanding(&self) -> Option<&Self> {
        self.failing.first().map(|_| self)
    }

    pub(crate) fn complained(&self, out: &mut impl Write) -> Result<(), Failure<'static>> {
        self.failing.iter().try_for_each(|change| writeln!(out, "{}", change.now_at().display()))?;
        Ok(writeln!(out, "::error::{}", self.fault)?)
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Beyond(Check::Goldens) => write!(
                formatter,
                "this PR changes goldens of crates that nothing else it changes reaches, so it did not produce them; only the owner passes this, with the label {}",
                Check::GOLDENS
            ),
            Self::Beyond(Check::Scenarios) => write!(
                formatter,
                "this PR adds files under tests/fixtures or tests/e2e of crates that nothing else it changes reaches; only the owner passes this, with the label {}",
                Check::SCENARIOS
            ),
            Self::Refactor => write!(
                formatter,
                "this PR's title starts with refactor:, and a refactor leaves behaviour unchanged, so it changes no golden; only the owner passes this, with the label {}",
                Check::GOLDENS
            ),
        }
    }
}
