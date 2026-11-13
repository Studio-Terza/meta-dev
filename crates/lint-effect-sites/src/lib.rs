mod crate_effects;

use code_model::Model;
use lint_core::{Headline, ModelLint, Report};

pub use crate_effects::CrateEffects;

#[derive(Debug)]
pub struct EffectSites;

#[derive(Debug)]
pub struct EffectsInPorts;

impl ModelLint for EffectsInPorts {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "effects on files, environment, clock, network, processes, randomness and stdio happen only in ports",
            ),
            CrateEffects::of_model(model).values().flat_map(CrateEffects::outside_ports).collect(),
        )
    }
}

impl ModelLint for EffectSites {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, effect sites in shipped code per crate (files, environment, clock, network, processes, randomness, stdio), counted everywhere, listed only outside ports; move each behind a port: a crate whose name ends in ports, or a module named ports",
            ),
            CrateEffects::of_model(model).iter().flat_map(|(name, found)| found.violations(name)).collect(),
        )
    }
}
