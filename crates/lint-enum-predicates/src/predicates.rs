use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Macro, ReturnType, Type};

use lint_core::{Macros, Site, SourceFile, Violation};
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
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(&function.sig),
                _ => None,
            })
            .filter(|signature| match &signature.output {
                ReturnType::Type(_, returned) => {
                    matches!(returned.as_ref(), Type::Path(path) if path.path.is_ident("bool"))
                }
                ReturnType::Default => false,
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
        let found = match (&block.trait_, self.known.kind(self.file, &self.module, &block.self_ty)) {
            (None, Kind::Enum) => self.predicates(block),
            _ => Vec::new(),
        };
        self.violations.extend(found);
        syn::visit::visit_item_impl(self, block);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}
