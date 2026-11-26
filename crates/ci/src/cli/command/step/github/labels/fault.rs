use std::fmt;

use super::change::Change;
use super::check::Check;

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
    pub(crate) fn new(fault: Fault, failing: Vec<&'change Change>) -> Owed<'change> {
        Owed { fault, failing }
    }

    pub(crate) fn outstanding(&self) -> Option<&'change Change> {
        self.failing.first().copied()
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Beyond(Check::Goldens) => write!(
                formatter,
                "this PR changes goldens of crates that nothing else it changes reaches, so it did not produce them; only the owner passes this, with the label {}",
                Check::GOLDENS
            ),
            Fault::Beyond(Check::Scenarios) => write!(
                formatter,
                "this PR adds files under tests/fixtures or tests/e2e of crates that nothing else it changes reaches; only the owner passes this, with the label {}",
                Check::SCENARIOS
            ),
            Fault::Refactor => write!(
                formatter,
                "this PR's title starts with refactor:, and a refactor leaves behaviour unchanged, so it changes no golden; only the owner passes this, with the label {}",
                Check::GOLDENS
            ),
        }
    }
}

impl fmt::Display for Owed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.failing.iter().try_for_each(|change| writeln!(formatter, "{}", change.now_at().display()))?;
        writeln!(formatter, "::error::{}", self.fault)
    }
}
