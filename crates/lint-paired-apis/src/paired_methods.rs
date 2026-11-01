use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ItemImpl, ItemMod, ItemTrait, TraitItem, Visibility};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name};

use super::{Finding, Pair};

pub struct PairedMethods<'file> {
    file: &'file SourceFile,
    found: Vec<Violation>,
}

impl PairedMethods<'_> {
    pub fn of_file(file: &SourceFile) -> Vec<Violation> {
        let mut scan = PairedMethods { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn public_methods(block: &ItemImpl) -> Vec<&Ident> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) if matches!(method.vis, Visibility::Public(_)) => Some(&method.sig.ident),
                _ => None,
            })
            .collect()
    }

    fn declared_methods(block: &ItemTrait) -> Vec<&Ident> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) => Some(&method.sig.ident),
                _ => None,
            })
            .collect()
    }

    fn pairs_among(&mut self, methods: &[&Ident]) {
        let found: Vec<Violation> = Pair::ALL
            .iter()
            .flat_map(|pair| methods.iter().filter_map(move |method| Some((method, pair.partner(method)?))))
            .filter(|(_, partner)| methods.iter().any(|other| partner == *other))
            .map(|(method, partner)| {
                let site = Site::new(self.file.path(), LineNumber::from(method.span()));
                Violation::new(site, Finding::new(Name::from(*method), partner))
            })
            .collect();
        self.found.extend(found);
    }
}

impl<'ast> Visit<'ast> for PairedMethods<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let methods = match (Walk::inherent_impls(block), Walk::outside_tests(&block.attrs)) {
            (Walk::Entered, Walk::Entered) => Self::public_methods(block),
            _ => Vec::new(),
        };
        self.pairs_among(&methods);
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        Walk::enter(&block.attrs, || self.pairs_among(&Self::declared_methods(block)));
    }
}
