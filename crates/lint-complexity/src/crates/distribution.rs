mod band;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Name;
use lint_placement::CrateName;

use super::function::Function;
use band::Band;

#[derive(Debug)]
pub(crate) struct Distribution<'crate_functions> {
    name: &'crate_functions CrateName,
    bands: BTreeMap<Band, Vec<&'crate_functions Function>>,
    hairiest: &'crate_functions Function,
    total: Vec<&'crate_functions Function>,
}

impl<'crate_functions> Distribution<'crate_functions> {
    pub(crate) fn listing(name: &'crate_functions CrateName, functions: &'crate_functions [Function]) -> Option<Self> {
        let hairiest = functions.iter().max_by_key(|function| function.score())?;
        let bands = functions.iter().fold(BTreeMap::new(), |mut bands: BTreeMap<Band, Vec<&Function>>, function| {
            bands.entry(Band::containing(function.score())).or_default().push(function);
            bands
        });
        Some(Distribution { name, bands, hairiest, total: functions.iter().collect() })
    }

    pub(crate) fn violation(&self) -> Violation {
        let site: &Site = self.hairiest.as_ref();
        Violation::new(site.clone(), self)
    }
}

impl fmt::Display for Distribution<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hairiest: &Name = self.hairiest.as_ref();
        write!(formatter, "crate {}: {} functions by complexity", self.name, self.total.len())?;
        self.bands.iter().try_for_each(|(band, members)| write!(formatter, ", {band}: {}", members.len()))?;
        write!(formatter, "; highest {} in {hairiest}", self.hairiest.score())
    }
}
