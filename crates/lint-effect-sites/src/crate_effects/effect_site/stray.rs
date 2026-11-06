use std::fmt;

use lint_core::{Finding, Fix};

use super::Confinement;
use crate::crate_effects::called::Called;
use crate::crate_effects::effect::Effect;

#[derive(Debug)]
pub(super) struct Stray<'site> {
    effect: Effect,
    called: &'site Called,
    confinement: Confinement,
}

impl<'site> Stray<'site> {
    pub(super) fn new(effect: Effect, called: &'site Called, confinement: Confinement) -> Stray<'site> {
        Stray { effect, called, confinement }
    }
}

impl fmt::Display for Stray<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.confinement {
            Confinement::Ports => write!(formatter, "{} effect outside ports: calls {}", self.effect, self.called),
            Confinement::Fillers => write!(formatter, "{} effect outside a filler: calls {}", self.effect, self.called),
        }
    }
}

impl Finding for Stray<'_> {
    fn fix(&self) -> Fix {
        match self.confinement {
            Confinement::Ports => {
                Fix::new("move the effect behind a port in a ports module and call the port from here")
            }
            Confinement::Fillers => {
                Fix::new("move the effect into a method of the impl <Trait>Filler block, where the port counts it")
            }
        }
    }
}
