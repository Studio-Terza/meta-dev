mod graph;

use std::path::Path;

use syntax::{ItemMod, ItemUse, Macro, MacroTree};

use lint_core::Extended;
use lint_core::{MacroAliases, Macros, SourceFile, Violation, Walk};
use lint_layout::{Name, Word};
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use graph::Graph;

#[derive(Debug)]
pub struct DeclaredGraphs<'run, 'scan> {
    file: &'run SourceFile,
    path: &'scan Path,
}

#[derive(Debug, Default)]
pub struct Seen {
    names: Option<MacroAliases>,
    gates: Vec<Walk>,
    invocations: Vec<Macro>,
}

impl<'run, 'scan> DeclaredGraphs<'run, 'scan> {
    const MACRO: &'static str = "states";

    pub fn new(file: &'run SourceFile, path: &'scan Path) -> DeclaredGraphs<'run, 'scan> {
        DeclaredGraphs { file, path }
    }

    fn names(found: &Seen) -> MacroAliases {
        found.names.clone().unwrap_or_else(|| MacroAliases::new(Word::new(DeclaredGraphs::MACRO)))
    }

    pub fn checked(&self, found: Seen) -> Vec<Violation<'scan>> {
        let names = Self::names(&found);
        found.invocations.into_iter().flat_map(|invoked| self.found_in(&invoked, &names)).collect()
    }

    fn found_in(&self, invocation: &Macro, names: &MacroAliases) -> Vec<Violation<'scan>> {
        let nested = |inner: &Macro| Macros::nested(self.file, inner).to_vec();
        let stops = |found: &Macro| {
            found.path.segments.last().and_then(|last| names.call(&Name::from(&last.ident))).map(|_| ())
        };
        let graphs = MacroTree::frontier(invocation, &nested, &stops);
        graphs.iter().flat_map(|found| Graph::checked(found, self.path)).collect()
    }
}

impl Fold for DeclaredGraphs<'_, '_> {
    type Found = Seen;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemUse, Kind::Macro]
    }

    fn on_item_mod(&self, found: Seen, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Seen> {
        Visit::descending(Seen { gates: found.gates.extended([facts.gate()]), ..found })
    }

    fn off_item_mod(&self, found: Seen, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Seen {
        let mut gates = found.gates;
        gates.pop();
        Seen { gates, ..found }
    }

    fn on_item_use(&self, found: Seen, item: &ItemUse, _: &Facts<'_, ItemUse>) -> Visit<Seen> {
        let names = Self::names(&found).used(item);
        Visit::new(Seen { names: Some(names), ..found }, Descend::Over)
    }

    fn on_macro(&self, found: Seen, invocation: &Macro, _: &Facts<'_, Macro>) -> Visit<Seen> {
        let outside = found.gates.iter().all(|gate| *gate == Walk::Entered).then(|| invocation.clone());
        Visit::new(Seen { invocations: found.invocations.extended(outside), ..found }, Descend::Over)
    }
}
