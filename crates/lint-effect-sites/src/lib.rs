mod crate_effects;

use std::collections::BTreeMap;

use graph::Model;
use lint_core::{Headline, ModelLint, Report};
use lint_placement::CrateName;

use crate_effects::CrateEffects;
pub use crate_effects::OutsideFillers;

#[derive(Debug)]
pub struct EffectSites<'model>(BTreeMap<CrateName, CrateEffects<'model>>);

#[derive(Debug)]
pub struct EffectsInPorts;

impl<'model> EffectSites<'model> {
    #[must_use]
    pub fn of_model(model: &'model Model) -> EffectSites<'model> {
        EffectSites(CrateEffects::of_model(model))
    }

    #[must_use]
    pub fn per_crate(&self) -> Report<'model> {
        Report::new(
            Headline::new(
                "report only, effect sites in shipped code per crate (files, environment, clock, network, processes, randomness, stdio), counted everywhere, listed only outside ports; move each behind a port: a crate whose name ends in ports, or a module named ports",
            ),
            self.0.iter().flat_map(|(name, found)| found.violations(name)).collect(),
        )
    }
}

impl ModelLint for EffectsInPorts {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "effects on files, environment, clock, network, processes, randomness and stdio happen only in ports",
            ),
            EffectSites::of_model(model).0.values().flat_map(CrateEffects::outside_ports).collect(),
        )
    }
}
