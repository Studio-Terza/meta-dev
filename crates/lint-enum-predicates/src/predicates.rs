use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemImpl, ItemMod, Macro};

use lint_core::{Macros, Site, SourceFile, Syntax, Violation};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;

use super::{EnumPredicates, Finding, Kind};

pub struct Predicates<'scan, 'file> {
    known: &'scan EnumPredicates<'scan>,
    file: &'file SourceFile,
    module: Cow<'file, ModulePath>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> Predicates<'scan, 'file> {
    pub fn new(known: &'scan EnumPredicates<'scan>, file: &'file SourceFile) -> Predicates<'scan, 'file> {
        Predicates { known, file, module: Cow::Borrowed(file.placement().module()), violations: Vec::new() }
    }

    pub fn into_violations(self) -> Vec<Violation<'file>> {
        self.violations
    }

    fn predicates(&self, block: &ItemImpl) -> Vec<Violation<'file>> {
        block
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .map(|function| &function.sig)
            .filter(|signature| {
                Syntax::returned(&signature.output)
                    .and_then(Syntax::type_path)
                    .is_some_and(|path| path.path.is_ident("bool"))
            })
            .map(|signature| {
                Violation::new(
                    Site::new(self.file.path(), LineNumber::from(signature.span())),
                    Finding::BoolPredicate { name: Name::from(&signature.ident) },
                )
            })
            .collect()
    }
}

impl<'ast> Visit<'ast> for Predicates<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = Cow::Owned(self.module.child(Name::from(&module.ident)));
        let outer = std::mem::replace(&mut self.module, inner);
        syn::visit::visit_item_mod(self, module);
        self.module = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let found = Syntax::inherent(block)
            .filter(|block| self.known.kind(self.file, &self.module, &block.self_ty) == Kind::Enum)
            .map(|block| self.predicates(block))
            .unwrap_or_default();
        self.violations.extend(found);
        syn::visit::visit_item_impl(self, block);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}
