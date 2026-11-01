mod graph;

use std::path::Path;

use syn::visit::Visit;
use syn::{ItemMod, Macro};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::LineNumber;

use super::finding::Finding;
use graph::Graph;

pub struct DeclaredGraphs<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl<'scan> DeclaredGraphs<'scan> {
    const MACRO: &'static str = "states";

    pub fn violations(file: &'scan SourceFile) -> Vec<Violation> {
        let mut graphs = DeclaredGraphs { path: file.path(), violations: Vec::new() };
        graphs.visit_file(file.syntax());
        graphs.violations
    }

    fn checked(&self, invocation: &Macro) -> Vec<Violation> {
        invocation.parse_body::<Graph>().map_or_else(
            |_| {
                let site = Site::new(self.path, LineNumber::from(invocation.bang_token.span));
                vec![Violation::new(site, Finding::UnreadableGraph)]
            },
            |graph| graph.violations(self.path),
        )
    }
}

impl<'ast> Visit<'ast> for DeclaredGraphs<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let found = Some(invocation)
            .filter(|invocation| invocation.path.segments.last().is_some_and(|last| last.ident == Self::MACRO))
            .map(|invocation| self.checked(invocation));
        self.violations.extend(found.into_iter().flatten());
    }
}
