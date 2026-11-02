mod band;

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::crate_name::CrateName;
use crate::cli::workspace::share::Share;
use band::Band;

#[derive(Debug)]
pub struct FanIn {
    total: CrateCount,
    by_count: BTreeMap<Reverse<CrateCount>, BTreeSet<CrateName>>,
}

impl FanIn {
    pub fn among(total: CrateCount) -> FanIn {
        FanIn { total, by_count: BTreeMap::new() }
    }

    pub fn inline(&self) -> Inline<'_> {
        Inline(self)
    }

    pub fn counted(mut self, count: CrateCount, name: CrateName) -> FanIn {
        self.by_count.entry(Reverse(count)).or_default().insert(name);
        self
    }

    pub fn share_of(&self, crate_name: &CrateName) -> Option<Share> {
        self.by_count
            .iter()
            .find_map(|(Reverse(count), names)| names.contains(crate_name).then(|| Share::new(*count, self.total)))
    }

    fn shares(&self) -> impl Iterator<Item = Share> {
        self.by_count
            .iter()
            .flat_map(move |(Reverse(count), names)| names.iter().map(move |_| Share::new(*count, self.total)))
    }

    fn distribution(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "crates per tenth of the workspace depending on them:")?;
        Band::all().try_for_each(|band| {
            writeln!(
                formatter,
                "{band} {}",
                self.shares().filter(|share| Band::holding(share.percent()) == band).count()
            )
        })?;
        let shares: Vec<Share> = self.shares().collect();
        let median = shares.get(shares.len() >> 1);
        let largest = self.by_count.iter().next();
        match (median, largest) {
            (Some(median), Some((Reverse(count), names))) => {
                let share = Share::new(*count, self.total);
                let names: Vec<String> = names.iter().map(ToString::to_string).collect();
                writeln!(
                    formatter,
                    "median {median}, {}%; largest {share}, {}%: {}",
                    median.percent(),
                    share.percent(),
                    names.join(" ")
                )
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Inline<'fan_in>(&'fan_in FanIn);

impl fmt::Display for Inline<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<String> = self
            .0
            .by_count
            .iter()
            .flat_map(|(Reverse(count), names)| names.iter().map(move |name| format!("{name}={count}")))
            .collect();
        write!(formatter, "{}", entries.join(" "))
    }
}

impl fmt::Display for FanIn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "fan-in, crates that depend on each crate, directly or not, of {} workspace crates:",
            self.total
        )?;
        self.by_count.iter().try_for_each(|(Reverse(count), names)| {
            let share = Share::new(*count, self.total);
            names.iter().try_for_each(|name| writeln!(formatter, "{share}, {}%: {name}", share.percent()))
        })?;
        self.distribution(formatter)
    }
}
