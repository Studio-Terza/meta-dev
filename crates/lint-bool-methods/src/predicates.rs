mod members;

use syn::visit::Visit;
use syn::{Ident, ItemImpl, ItemMod, ItemTrait, ReturnType, Signature, Type, Visibility};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name, Prefix};

use super::Finding;
use members::Members;

pub struct Predicates<'file> {
    file: &'file SourceFile,
    found: Vec<Violation>,
}

impl Predicates<'_> {
    const PREFIXES: [&'static str; 2] = ["is_", "has_"];

    pub fn of_file(file: &SourceFile) -> Vec<Violation> {
        let mut scan = Predicates { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn in_block(&self, block: &ItemImpl) -> Vec<Violation> {
        match (Walk::inherent_impls(block), Walk::outside_tests(&block.attrs)) {
            (Walk::Entered, Walk::Entered) => {
                Members::public_in(block).into_iter().filter_map(|signature| self.violation(signature)).collect()
            }
            _ => Vec::new(),
        }
    }

    fn in_trait(&self, block: &ItemTrait) -> Vec<Violation> {
        match (Walk::outside_tests(&block.attrs), &block.vis) {
            (Walk::Entered, Visibility::Public(_)) => {
                Members::declared_in(block).into_iter().filter_map(|signature| self.violation(signature)).collect()
            }
            _ => Vec::new(),
        }
    }

    fn predicate(signature: &Signature) -> Option<&Ident> {
        let returns_bool = match &signature.output {
            ReturnType::Type(_, returned) => {
                matches!(returned.as_ref(), Type::Path(path) if path.path.is_ident("bool"))
            }
            ReturnType::Default => false,
        };
        (returns_bool && Name::from(&signature.ident).matches::<Prefix>(&Self::PREFIXES)).then_some(&signature.ident)
    }

    fn violation(&self, signature: &Signature) -> Option<Violation> {
        Self::predicate(signature)
            .map(|name| Violation::new(Site::new(self.file.path(), LineNumber::from(name.span())), Finding::from(name)))
    }
}

impl<'ast> Visit<'ast> for Predicates<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        self.found.extend(self.in_block(block));
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        self.found.extend(self.in_trait(block));
    }
}
