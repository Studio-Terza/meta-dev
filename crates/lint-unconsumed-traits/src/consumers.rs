use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{ExprMethodCall, ExprPath, ItemMod, TraitBound};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Consumers {
    bounds: BTreeSet<Name>,
    called: BTreeSet<Name>,
}

impl Consumers {
    pub(crate) fn of_files<'file>(files: impl Iterator<Item = &'file SourceFile>) -> Self {
        files.fold(Self::default(), |mut scan, file| {
            scan.visit_file(file.syntax());
            scan
        })
    }

    pub(crate) fn consuming<'name>(&'name self, name: &'name Name, provided: &'name [Name]) -> Option<&'name Name> {
        self.bounds.get(name).or_else(|| provided.iter().find(|method| self.called.contains(*method)))
    }
}

impl<'ast> Visit<'ast> for Consumers {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_trait_bound(&mut self, bound: &'ast TraitBound) {
        self.bounds.extend(bound.path.segments.last().map(|segment| Name::from(&segment.ident)));
        visit::visit_trait_bound(self, bound);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.called.insert(Name::from(&call.method));
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.called.extend(
            path.path
                .segments
                .iter()
                .nth(1)
                .and_then(|_| path.path.segments.last())
                .map(|segment| Name::from(&segment.ident)),
        );
        visit::visit_expr_path(self, path);
    }
}
