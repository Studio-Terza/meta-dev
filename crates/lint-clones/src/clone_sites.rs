mod finding;

use syn::visit::{self, Visit};
use syn::{ExprMethodCall, ItemMod};

use lint_core::{Gate, Site, SourceFile, Violation, Walk};
use lint_layout::LineNumber;

use finding::Finding;

pub struct CloneSites<'file> {
    file: &'file SourceFile,
    found: Vec<Violation<'file>>,
}

impl<'file> CloneSites<'file> {
    const CLONE: &str = "clone";

    pub fn within(file: &'file SourceFile) -> std::vec::IntoIter<Violation<'file>> {
        let mut sites = CloneSites { file, found: Vec::new() };
        sites.visit_file(file.syntax());
        sites.found.into_iter()
    }
}

impl<'ast> Visit<'ast> for CloneSites<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Gate::BOUNDARY.names(module) {
            Some(_) => {}
            None => Walk::enter(&module.attrs, || visit::visit_item_mod(self, module)),
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let cloned = (call.method == Self::CLONE && call.args.is_empty())
            .then(|| Violation::new(Site::new(self.file.path(), LineNumber::from(call.method.span())), Finding));
        self.found.extend(cloned);
        visit::visit_expr_method_call(self, call);
    }
}
