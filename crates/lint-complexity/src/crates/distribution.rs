mod band;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Finding, Fix, Site, Violation};
use lint_layout::Name;
use lint_placement::CrateName;

use super::function::Function;
use band::Band;

#[derive(Debug)]
pub(crate) struct Distribution<'file, 'crate_functions> {
    name: &'crate_functions CrateName,
    bands: BTreeMap<Band, Vec<&'crate_functions Function<'file>>>,
    hairiest: &'crate_functions Function<'file>,
    total: Vec<&'crate_functions Function<'file>>,
}

impl<'file, 'crate_functions> Distribution<'file, 'crate_functions> {
    pub(crate) fn listing(
        name: &'crate_functions CrateName,
        functions: &'crate_functions [Function<'file>],
    ) -> Option<Self> {
        let hairiest = functions.iter().max_by_key(|function| function.score())?;
        let bands =
            functions.iter().fold(BTreeMap::new(), |mut bands: BTreeMap<Band, Vec<&Function<'file>>>, function| {
                bands.entry(Band::containing(function.score())).or_default().push(function);
                bands
            });
        Some(Distribution { name, bands, hairiest, total: functions.iter().collect() })
    }

    pub(crate) fn violation(&self) -> Violation<'file> {
        let site: &Site<'file> = self.hairiest.as_ref();
        Violation::new(site.clone(), self)
    }
}

impl<'file> fmt::Display for Distribution<'file, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hairiest: &Name = self.hairiest.as_ref();
        write!(formatter, "crate {}: {} functions by complexity", self.name, self.total.len())?;
        self.bands.iter().try_for_each(|(band, members)| write!(formatter, ", {band}: {}", members.len()))?;
        write!(formatter, ", highest {} in {hairiest}", self.hairiest.score())
    }
}

impl<'file> Finding for Distribution<'file, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "turn the branches of the highest into a standard trait, a generic over an existing trait, or a derive",
        )
    }
}
