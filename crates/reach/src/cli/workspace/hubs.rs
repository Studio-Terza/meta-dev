mod among;
mod hub;

use std::fmt;

use lint_complexity::Logic;
use lint_core::{Headline, Report, Violation};
use newtype::{Counter, Newtype};

use crate::cli::workspace::percent::Percent;
use among::Among;
pub use hub::Hub;

#[derive(Debug)]
pub struct Hubs<'file> {
    hubs: Vec<Hub<'file>>,
}

impl<'file> Hubs<'file> {
    const HEADLINE: &'static str = "report only, hub thickness: the logic each crate holds, its shipped functions' cyclomatic complexity above one summed, weighted by the crates that depend on it; a crate many depend on should hold types, traits and interfaces, with the logic in the leaves";

    pub fn weighed(hubs: Vec<Hub<'file>>) -> Hubs<'file> {
        Hubs { hubs: Hub::ranked(hubs) }
    }

    fn logic(&self, among: Among) -> Logic {
        Logic::total(self.hubs.iter().filter_map(|hub| hub.counted(among)))
    }
}

impl<'file> fmt::Display for Hubs<'file> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let violations: Vec<Violation<'file>> = self.hubs.iter().map(Violation::from).collect();
        write!(formatter, "{}", Report::new(Headline::new(Self::HEADLINE), violations))?;
        let held = self.logic(Among::Majority);
        let all = self.logic(Among::All);
        let percent =
            Percent::new((held.into_inner() * Percent::WHOLE).checked_div(all.into_inner()).unwrap_or_default());
        writeln!(formatter, "logic in crates more than half the workspace depends on: {held} of {all}, {percent}%")
    }
}
