mod finding;

use syn::visit::{self, Visit};
use syn::{ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Signature, TraitItemFn};

use lint_core::{Site, SourceFile, Syntax, Violation, Walk};
use lint_file::FileLint;
use lint_layout::LineNumber;

use finding::Finding;

pub struct Returns<'file> {
    file: &'file SourceFile,
    found: Vec<Violation<'file>>,
}

impl<'file> Returns<'file> {
    fn flagged(&self, signature: &Signature) -> Option<Violation<'file>> {
        Syntax::returned(&signature.output)
            .and_then(Syntax::type_path)
            .filter(|path| path.qself.is_none())
            .filter(|path| path.path.is_ident("bool"))
            .map(|_| {
                Violation::new(
                    Site::new(self.file.path(), LineNumber::from(signature.ident.span())),
                    Finding::from(&signature.ident),
                )
            })
    }

    fn record(&mut self, signature: &Signature) {
        let flagged = self.flagged(signature);
        self.found.extend(flagged);
    }
}

impl FileLint for Returns<'_> {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut scan = Returns { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }
}

impl<'ast> Visit<'ast> for Returns<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || {
            self.record(&function.sig);
            visit::visit_item_fn(self, function);
        });
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        match Walk::inherent_impls(block) {
            Walk::Entered => Walk::enter(&block.attrs, || visit::visit_item_impl(self, block)),
            Walk::Skipped => {}
        }
    }

    fn visit_impl_item_fn(&mut self, method: &'ast ImplItemFn) {
        Walk::enter(&method.attrs, || {
            self.record(&method.sig);
            visit::visit_impl_item_fn(self, method);
        });
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        Walk::enter(&block.attrs, || visit::visit_item_trait(self, block));
    }

    fn visit_trait_item_fn(&mut self, method: &'ast TraitItemFn) {
        Walk::enter(&method.attrs, || {
            self.record(&method.sig);
            visit::visit_trait_item_fn(self, method);
        });
    }
}
