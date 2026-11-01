mod graph;

use std::path::Path;

use syn::visit::Visit;
use syn::{ItemMod, Macro};

use lint_core::{MacroAliases, MacroCalls, SourceFile, Violation, Walk};
use lint_layout::{Name, Word};

use graph::Graph;

pub struct DeclaredGraphs<'scan> {
    path: &'scan Path,
    names: MacroAliases,
    violations: Vec<Violation>,
}

impl<'scan> DeclaredGraphs<'scan> {
    const MACRO: &'static str = "states";

    pub fn violations(file: &'scan SourceFile) -> Vec<Violation> {
        let mut graphs = DeclaredGraphs {
            path: file.path(),
            names: MacroAliases::in_file(file.syntax(), Word::new(Self::MACRO)),
            violations: Vec::new(),
        };
        graphs.visit_file(file.syntax());
        graphs.violations
    }

    fn found_in(&self, invocation: &Macro) -> Vec<Violation> {
        if invocation.path.segments.last().is_some_and(|last| self.names.call(&Name::from(&last.ident))) {
            Graph::checked(invocation, self.path)
        } else {
            MacroCalls::within(invocation).iter().flat_map(|inner| self.found_in(inner)).collect()
        }
    }
}

impl<'ast> Visit<'ast> for DeclaredGraphs<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let found = self.found_in(invocation);
        self.violations.extend(found);
    }
}
