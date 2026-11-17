use std::collections::BTreeMap;
use std::fmt;

use graph::{PathUse, Placed};
use lint_core::{Finding, Fix, Site, Violation};
use lint_layout::{Length, Name};

use super::Panics;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Named<'model> {
    krate: &'model Name,
    site: Site<'model>,
    count: Length,
}

impl<'model> Named<'model> {
    pub(crate) fn per_crate(uses: impl Iterator<Item = Placed<'model, &'model PathUse>>) -> Vec<Named<'model>> {
        let crates = uses.filter(|placed| Panics::named(placed).is_some()).fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<&Name, Vec<Site<'model>>>, placed| {
                crates.entry(placed.krate()).or_default().push(placed.site());
                crates
            },
        );
        crates
            .into_iter()
            .filter_map(|(krate, sites)| {
                let site = sites.iter().min().copied()?;
                Some(Named { krate, site, count: Length::new(sites.len()) })
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
        write!(formatter, "crate {}: {} named invariants", self.krate, self.count)
    }
}

impl Finding for Named<'_> {
    fn fix(&self) -> Fix {
        Fix::new("report only: each named invariant marks a requirement a refinement type could carry instead")
    }
}
