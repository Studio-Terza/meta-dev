mod graph;

use std::path::Path;

use syntax::{ItemMod, ItemUse, Macro, MacroTree};

use algebra::{Monoid, Semigroup};
use lint_core::{Aliases, MacroAliases, Macros, SourceFile, Violation, Walk};
use lint_layout::{Name, Word};
use lint_sweep::{Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope};

use graph::Graph;

#[derive(Debug)]
pub struct DeclaredGraphs<'run, 'scan> {
    file: &'run SourceFile,
    path: &'scan Path,
    names: MacroAliases,
}

#[derive(Debug, Default)]
pub struct Invoked {
    aliases: Aliases,
    invocations: Vec<Macro>,
}

impl Semigroup for Invoked {
    fn combine(self, other: Invoked) -> Invoked {
        Invoked {
            aliases: self.aliases.combine(other.aliases),
            invocations: self.invocations.combine(other.invocations),
        }
    }
}

impl Monoid for Invoked {}

impl Invoked {
    fn gated(self, gate: Walk) -> Invoked {
        let invocations = Descend::from(gate).entered().map_or_else(Vec::new, |_| self.invocations);
        Invoked { aliases: self.aliases, invocations }
    }
}

impl<'run, 'scan> DeclaredGraphs<'run, 'scan> {
    const MACRO: &'static str = "states";

    pub fn new(file: &'run SourceFile, path: &'scan Path) -> DeclaredGraphs<'run, 'scan> {
        DeclaredGraphs { file, path, names: MacroAliases::new(Word::new(DeclaredGraphs::MACRO)) }
    }

    pub fn found(self, invoked: Invoked) -> Vec<Violation<'scan>> {
        let Invoked { aliases, invocations } = invoked;
        let DeclaredGraphs { file, path, names: macros } = self;
        let names = macros.aliased(aliases);
        let nested = |inner: &Macro| Macros::nested(file, inner).to_vec();
        let stops = |found: &Macro| {
            found.path.segments.last().and_then(|last| names.call(&Name::from(&last.ident))).map(|_| ())
        };
        let graphs = invocations.iter().flat_map(|invocation| MacroTree::frontier(invocation, &nested, &stops));
        graphs.flat_map(|found| Graph::checked(&found, path)).collect()
    }
}

impl Contribute for DeclaredGraphs<'_, '_> {
    type Open = ();
    type Part = Invoked;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemUse, Kind::Macro]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemMod)] }
    }

    fn off_item_mod(&self, _: &ItemMod, facts: &Facts<'_, ItemMod>, inner: Contribution<(), Invoked>) -> Invoked {
        inner.concluded(|(), invoked| invoked.gated(facts.gate()))
    }

    fn on_item_use(
        &self,
        item: &ItemUse,
        _: &Facts<'_, ItemUse>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Invoked> {
        let aliases = self.names.renamed(&item.tree);
        Contributed::new((), Invoked { aliases, invocations: Vec::new() }, Descend::Over)
    }

    fn on_macro(
        &self,
        invocation: &Macro,
        _: &Facts<'_, Macro>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Invoked> {
        Contributed::new(
            (),
            Invoked { aliases: Aliases::default(), invocations: vec![invocation.clone()] },
            Descend::Over,
        )
    }
}
