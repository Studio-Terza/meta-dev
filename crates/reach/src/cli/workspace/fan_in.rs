use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::crate_name::CrateName;

#[derive(Debug, Default)]
pub struct FanIn {
    by_count: BTreeMap<Reverse<CrateCount>, BTreeSet<CrateName>>,
}

impl FanIn {
    pub fn counted(mut self, count: CrateCount, name: CrateName) -> FanIn {
        self.by_count.entry(Reverse(count)).or_default().insert(name);
        self
    }
}

impl fmt::Display for FanIn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "fan-in, crates that depend on each crate, directly or not:")?;
        self.by_count.iter().try_for_each(|(Reverse(count), names)| {
            names.iter().try_for_each(|name| writeln!(formatter, "{count} {name}"))
        })
    }
}
