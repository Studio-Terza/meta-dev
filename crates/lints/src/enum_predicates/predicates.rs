use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Macro, ReturnType, Type};

use lint_core::{LineNumber, Macros, ModulePath, Name, Site, SourceFile, Violation};

use super::{EnumPredicates, Finding};

pub struct Predicates<'scan> {
    known: &'scan EnumPredicates,
    file: &'scan SourceFile,
    module: ModulePath,
    violations: Vec<Violation>,
}

impl<'scan> Predicates<'scan> {
    pub fn new(known: &'scan EnumPredicates, file: &'scan SourceFile) -> Predicates<'scan> {
        Predicates { known, file, module: file.module().clone(), violations: Vec::new() }
    }

    pub fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn predicates(&self, block: &ItemImpl) -> Vec<Violation> {
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

impl<'ast> Visit<'ast> for Predicates<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = self.module.child(&Name::from(&module.ident));
        let outer = std::mem::replace(&mut self.module, inner);
        syn::visit::visit_item_mod(self, module);
        self.module = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        if block.trait_.is_none() && self.known.is_enum(self.file, &self.module, &block.self_ty) {
            let found = self.predicates(block);
            self.violations.extend(found);
        }
        syn::visit::visit_item_impl(self, block);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        for statement in Macros::statements(invocation) {
            self.visit_stmt(&statement);
        }
    }
}
