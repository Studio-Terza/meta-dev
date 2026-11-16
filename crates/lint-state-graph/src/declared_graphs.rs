mod graph;

use std::path::Path;

use syntax::{ItemMod, ItemUse, Macro, MacroTree};

use lint_core::{MacroAliases, Macros, SourceFile, Violation, Walk};
use lint_layout::{Name, Word};
use lint_sweep::{Descend, Facts, Kind, Watch};

use graph::Graph;

#[derive(Debug)]
pub struct DeclaredGraphs<'run, 'scan> {
    file: &'run SourceFile,
    path: &'scan Path,
    names: MacroAliases,
    gates: Vec<Walk>,
    invocations: Vec<Macro>,
}

impl<'run, 'scan> DeclaredGraphs<'run, 'scan> {
    const MACRO: &'static str = "states";

    pub fn new(file: &'run SourceFile, path: &'scan Path) -> DeclaredGraphs<'run, 'scan> {
        DeclaredGraphs {
            file,
            path,
            names: MacroAliases::new(Word::new(DeclaredGraphs::MACRO)),
            gates: Vec::new(),
            invocations: Vec::new(),
        }
    }

    fn found_in(&self, invocation: &Macro) -> Vec<Violation<'scan>> {
        let nested = |inner: &Macro| Macros::nested(self.file, inner).to_vec();
        let stops = |found: &Macro| {
            found.path.segments.last().and_then(|last| self.names.call(&Name::from(&last.ident))).map(|_| ())
        };
        let graphs = MacroTree::frontier(invocation, &nested, &stops);
        graphs.iter().flat_map(|found| Graph::checked(found, self.path)).collect()
    }
}

impl<'ast> Watch<'ast> for DeclaredGraphs<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemUse, Kind::Macro]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        self.gates.push(facts.gate());
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod, _: &Facts<'_, ItemMod>) {
        self.gates.pop();
    }

    fn on_item_use(&mut self, item: &'ast ItemUse, facts: &Facts<'_, ItemUse>) -> Descend {
        self.names.on_item_use(item, facts)
    }

    fn on_macro(&mut self, invocation: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        let outside = self.gates.iter().all(|gate| *gate == Walk::Entered).then(|| invocation.clone());
        self.invocations.extend(outside);
        Descend::Over
    }
}

impl<'file> From<DeclaredGraphs<'_, 'file>> for Vec<Violation<'file>> {
    fn from(scan: DeclaredGraphs<'_, 'file>) -> Vec<Violation<'file>> {
        scan.invocations.iter().flat_map(|invoked| scan.found_in(invoked)).collect()
    }
}
