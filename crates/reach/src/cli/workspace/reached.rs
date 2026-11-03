use std::collections::BTreeSet;
use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::crate_name::CrateName;
use crate::cli::workspace::share::Share;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cargo {
    Reached,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flag {
    Cargo,
    Mutants,
}

impl fmt::Display for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Flag::Cargo => "-p ",
            Flag::Mutants => "--test-package=",
        })
    }
}

#[derive(Debug)]
pub struct Reached<'meta> {
    crates: BTreeSet<&'meta CrateName>,
    total: CrateCount,
    cargo: Cargo,
}

impl<'meta> Reached<'meta> {
    const WORKSPACE_FLAG: &'static str = "--workspace";

    pub fn new(crates: BTreeSet<&'meta CrateName>, total: CrateCount) -> Reached<'meta> {
        Reached { crates, total, cargo: Cargo::Reached }
    }

    #[must_use]
    pub fn widened(self) -> Reached<'meta> {
        Reached { cargo: Cargo::Workspace, ..self }
    }

    pub fn share(&self) -> Share {
        Share::new(CrateCount::new(self.crates.len()), self.total)
    }

    pub fn covering(&self, crates: &BTreeSet<&CrateName>) -> Option<&Reached<'meta>> {
        crates.is_subset(&self.crates).then_some(self)
    }

    fn build_scope(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cargo {
            Cargo::Reached => self.flagged(formatter, Flag::Cargo),
            Cargo::Workspace => formatter.write_str(Self::WORKSPACE_FLAG),
        }
    }

    fn flagged(&self, formatter: &mut fmt::Formatter<'_>, flag: Flag) -> fmt::Result {
        let flags: Vec<String> = self.crates.iter().map(|name| format!("{flag}{name}")).collect();
        formatter.write_str(&flags.join(" "))
    }
}

impl fmt::Display for Reached<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "REACH={} of {} crates", self.crates.len(), self.total)?;
        write!(formatter, "BUILD_SCOPE=")?;
        self.build_scope(formatter)?;
        write!(formatter, "\nMUTANTS_SCOPE=")?;
        self.flagged(formatter, Flag::Mutants)?;
        writeln!(formatter)
    }
}
