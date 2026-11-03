use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ItemImpl, ItemMod, ItemTrait, TraitItem, Visibility};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_paths::Module;

use crate::declared::Declared;
use crate::owner::Owner;

pub struct PairedMethods<'file> {
    file: &'file SourceFile,
    module: Module<'file>,
    found: BTreeMap<Owner<'file>, Vec<Declared<'file>>>,
}

impl<'file> PairedMethods<'file> {
    pub fn of_files(files: &'file [SourceFile]) -> BTreeMap<Owner<'file>, Vec<Declared<'file>>> {
        files.iter().filter_map(SourceFile::shipped).fold(BTreeMap::new(), |found, file| {
            let mut scan = PairedMethods { file, module: Module::of_file(file), found };
            scan.visit_file(file.syntax());
            scan.found
        })
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

    fn owned_by(&mut self, owner: Option<Name>, methods: &[&Ident]) {
        let path = self.file.path();
        let declared = methods
            .iter()
            .map(|method| Declared::new(Name::from(*method), Site::new(path, LineNumber::from(method.span()))));
        owner.zip(Some(declared)).into_iter().for_each(|(name, declared)| {
            self.found.entry(Owner::new(&self.module, name)).or_default().extend(declared);
        });
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
        self.owned_by(Name::of_type(&block.self_ty), &methods);
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        Walk::enter(&block.attrs, || self.owned_by(Some(Name::from(&block.ident)), &Self::declared_methods(block)));
    }
}
