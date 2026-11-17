use std::collections::BTreeMap;
use std::fmt;

use graph::{Call, PathUse, Placed};
use lint_core::{Finding, Fix, Site, Violation};
use lint_layout::{Length, Name};

use super::Panics;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Named<'model> {
    krate: &'model Name,
    site: Site<'model>,
    invariants: Length,
    ignored: Length,
}

#[derive(Debug, Default)]
struct Sites<'model> {
    invariants: Vec<Site<'model>>,
    ignored: Vec<Site<'model>>,
}

impl<'model> Named<'model> {
    pub(crate) fn per_crate(
        uses: impl Iterator<Item = Placed<'model, &'model PathUse>>,
        calls: impl Iterator<Item = Placed<'model, &'model Call>>,
    ) -> Vec<Named<'model>> {
        let mut crates: BTreeMap<&'model Name, Sites<'model>> = BTreeMap::new();
        uses.filter(|placed| Panics::named(placed).is_some())
            .for_each(|placed| crates.entry(placed.krate()).or_default().invariants.push(placed.site()));
        calls
            .filter(|placed| Panics::ignoring(placed).is_some())
            .for_each(|placed| crates.entry(placed.krate()).or_default().ignored.push(placed.site()));
        crates
            .into_iter()
            .filter_map(|(krate, sites)| {
                let site = sites.invariants.iter().chain(&sites.ignored).min().copied()?;
                Some(Named {
                    krate,
                    site,
                    invariants: Length::new(sites.invariants.len()),
                    ignored: Length::new(sites.ignored.len()),
                })
            })
            .collect()
    }
}

impl<'model> From<Named<'model>> for Violation<'model> {
    fn from(named: Named<'model>) -> Violation<'model> {
        Violation::new(named.site, named)
    }
}

impl fmt::Display for Named<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "crate {}: {} named invariants, {} errors ignored with a named reason",
            self.krate, self.invariants, self.ignored
        )
    }
}

impl Finding for Named<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "report only: each named invariant or ignored error marks a requirement a refinement type could carry instead",
        )
    }
}
