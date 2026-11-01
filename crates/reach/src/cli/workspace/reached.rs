use std::collections::BTreeSet;
use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::crate_name::CrateName;

#[derive(Debug)]
pub struct Reached {
    crates: BTreeSet<CrateName>,
    total: CrateCount,
}

impl Reached {
    const CARGO_FLAG: &'static str = "-p ";
    const MUTANTS_FLAG: &'static str = "--test-package=";

    pub fn new(crates: BTreeSet<CrateName>, total: CrateCount) -> Reached {
        Reached { crates, total }
    }

    fn flags(&self, flag: &str) -> String {
        self.crates.iter().map(|name| format!("{flag}{name}")).collect::<Vec<String>>().join(" ")
    }
}

impl fmt::Display for Reached {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "REACH={} of {} crates", self.crates.len(), self.total)?;
        writeln!(formatter, "CARGO_SCOPE={}", self.flags(Self::CARGO_FLAG))?;
        writeln!(formatter, "MUTANTS_SCOPE={}", self.flags(Self::MUTANTS_FLAG))
    }
}
