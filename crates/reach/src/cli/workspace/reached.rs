use std::collections::BTreeSet;
use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::crate_name::CrateName;

#[derive(Debug, Clone, Copy)]
pub struct Share {
    reached: CrateCount,
    total: CrateCount,
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.reached, self.total)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cargo {
    Reached,
    Workspace,
}

#[derive(Debug)]
pub struct Reached {
    crates: BTreeSet<CrateName>,
    total: CrateCount,
    cargo: Cargo,
}

impl Reached {
    const CARGO_FLAG: &'static str = "-p ";
    const MUTANTS_FLAG: &'static str = "--test-package=";
    const WORKSPACE_FLAG: &'static str = "--workspace";

    pub fn new(crates: BTreeSet<CrateName>, total: CrateCount) -> Reached {
        Reached { crates, total, cargo: Cargo::Reached }
    }

    #[must_use]
    pub fn widened(self) -> Reached {
        Reached { cargo: Cargo::Workspace, ..self }
    }

    pub fn share(&self) -> Share {
        Share { reached: CrateCount::new(self.crates.len()), total: self.total }
    }

    fn flags(&self, flag: &str) -> String {
        self.crates.iter().map(|name| format!("{flag}{name}")).collect::<Vec<String>>().join(" ")
    }
}

impl fmt::Display for Reached {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "REACH={} of {} crates", self.crates.len(), self.total)?;
        let scope = match self.cargo {
            Cargo::Reached => self.flags(Self::CARGO_FLAG),
            Cargo::Workspace => Self::WORKSPACE_FLAG.to_owned(),
        };
        writeln!(formatter, "BUILD_SCOPE={scope}")?;
        writeln!(formatter, "MUTANTS_SCOPE={}", self.flags(Self::MUTANTS_FLAG))
    }
}
