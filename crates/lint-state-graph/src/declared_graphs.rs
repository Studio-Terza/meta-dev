mod graph;

use std::path::Path;

use syn::{ItemMod, ItemUse, Macro};

use lint_core::{MacroAliases, MacroCalls, Violation, Walk};
use lint_layout::{Name, Word};
use lint_sweep::{Descend, Kind, Watch};

use graph::Graph;

#[derive(Debug)]
pub struct DeclaredGraphs<'scan> {
    path: &'scan Path,
    names: MacroAliases,
    gates: Vec<Walk>,
    invocations: Vec<Macro>,
}

impl<'scan> DeclaredGraphs<'scan> {
    const MACRO: &'static str = "states";

    pub fn new(path: &'scan Path) -> DeclaredGraphs<'scan> {
        DeclaredGraphs {
            path,
            names: MacroAliases::new(Word::new(DeclaredGraphs::MACRO)),
            gates: Vec::new(),
            invocations: Vec::new(),
        }
    }

    fn found_in(&self, invocation: &Macro) -> Vec<Violation<'scan>> {
        match invocation.path.segments.last().and_then(|last| self.names.call(&Name::from(&last.ident))) {
            Some(_) => Graph::checked(invocation, self.path),
            None => MacroCalls::within(invocation).iter().flat_map(|inner| self.found_in(inner)).collect(),
        }
    }
}

impl<'ast> Watch<'ast> for DeclaredGraphs<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemUse, Kind::Macro]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        self.gates.push(Walk::outside_tests(&module.attrs));
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.gates.pop();
    }

    fn on_item_use(&mut self, item: &'ast ItemUse) -> Descend {
        self.names.on_item_use(item)
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        let outside = self.gates.iter().all(|gate| *gate == Walk::Entered).then(|| invocation.clone());
        self.invocations.extend(outside);
        Descend::Over
    }
}

impl<'file> From<DeclaredGraphs<'file>> for Vec<Violation<'file>> {
    fn from(scan: DeclaredGraphs<'file>) -> Vec<Violation<'file>> {
        scan.invocations.iter().flat_map(|invocation| scan.found_in(invocation)).collect()
    }
}
