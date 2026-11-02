use std::fmt;

use lint_core::{Site, Violation};
use lint_paths::Ground;

use super::called::Called;
use super::effect::Effect;

#[derive(Clone, Debug)]
pub(super) struct EffectSite {
    site: Site,
    effect: Effect,
    called: Called,
    ground: Ground,
}

impl EffectSite {
    pub(super) fn new(site: Site, effect: Effect, called: Called, ground: Ground) -> EffectSite {
        EffectSite { site, effect, called, ground }
    }

    pub(super) fn place(&self) -> &Site {
        &self.site
    }

    pub(super) fn effect(&self) -> Effect {
        self.effect
    }

    pub(super) fn ground(&self) -> Ground {
        self.ground
    }

    pub(super) fn outside_ports(&self) -> Option<Violation> {
        (self.ground == Ground::Open).then(|| Violation::new(self.site.clone(), self))
    }
}

impl fmt::Display for EffectSite {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} effect outside ports: calls {}", self.effect, self.called)
    }
}
