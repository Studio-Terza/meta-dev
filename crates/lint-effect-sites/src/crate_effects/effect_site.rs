mod stray;

use lint_core::{Site, Violation};
use code_model::Ground;

use super::called::Called;
use super::effect::Effect;
use stray::Stray;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Confinement {
    Ports,
    Fillers,
}

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

    pub(super) fn violation(&self, confinement: Confinement) -> Violation<'file> {
        Violation::new(self.site, Stray::new(self.effect, &self.called, confinement))
    }
}
