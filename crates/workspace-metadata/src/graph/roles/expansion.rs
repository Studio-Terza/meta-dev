use std::collections::BTreeSet;

use crate::crate_name::CrateName;
use crate::metadata::{Expanders, PortTable};

use super::super::Graph;
use super::super::unit::Unit;

#[derive(Debug)]
pub struct Expansion<'meta> {
    port: &'meta CrateName,
    expanders: &'meta Expanders,
    unit: BTreeSet<&'meta CrateName>,
    declaring: BTreeSet<&'meta CrateName>,
}

impl<'meta> Expansion<'meta> {
    pub(in crate::graph) fn found(graph: &Graph<'meta>, units: &[Unit<'meta>]) -> Vec<Expansion<'meta>> {
        units
            .iter()
            .flat_map(|unit| {
                unit.declarations().values().filter_map(move |declared| Self::declared(graph, unit, declared))
            })
            .collect()
    }

    fn declared(graph: &Graph<'meta>, unit: &Unit<'meta>, declared: &'meta PortTable) -> Option<Expansion<'meta>> {
        let expanders = declared.expanders()?;
        let declaring = graph.declaring(expanders.role());
        Some(Expansion {
            port: graph.package(unit.port())?.name(),
            expanders,
            unit: unit.members().iter().filter_map(|id| Some(graph.package(id)?.name())).collect(),
            declaring: declaring.iter().filter_map(|id| Some(graph.package(id)?.name())).collect(),
        })
    }

    #[must_use]
    pub fn port(&self) -> &'meta CrateName {
        self.port
    }

    #[must_use]
    pub fn expanders(&self) -> &'meta Expanders {
        self.expanders
    }

    #[must_use]
    pub fn admits(&self, expander: &CrateName) -> Option<&'meta CrateName> {
        self.unit.get(expander).or_else(|| self.declaring.get(expander)).copied()
    }
}
