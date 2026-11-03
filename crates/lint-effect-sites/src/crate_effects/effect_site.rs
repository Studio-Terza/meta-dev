use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use lint_paths::Ground;

use super::called::Called;
use super::effect::Effect;

#[derive(Clone, Debug)]
pub(super) struct EffectSite<'file> {
    site: Site<'file>,
    effect: Effect,
    called: Called,
    ground: Ground,
}

impl<'file> EffectSite<'file> {
    pub(super) fn new(site: Site<'file>, effect: Effect, called: Called, ground: Ground) -> EffectSite<'file> {
        EffectSite { site, effect, called, ground }
    }

    pub(super) fn place(&self) -> Site<'file> {
        self.site
    }

    pub(super) fn effect(&self) -> Effect {
        self.effect
    }

    pub(super) fn ground(&self) -> Ground {
        self.ground
    }

    pub(super) fn outside_ports(&self) -> Option<Violation<'file>> {
        (self.ground == Ground::Open).then(|| Violation::new(self.site, self))
    }
}

impl fmt::Display for EffectSite<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} effect outside ports: calls {}", self.effect, self.called)
    }
}

impl lint_layout::Finding for EffectSite<'_> {
    fn fix(&self) -> Fix {
        Fix::new("move the effect behind a port in a ports module and call the port from here")
    }
}
