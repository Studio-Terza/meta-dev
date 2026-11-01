use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ItemImpl, ItemMod, ItemTrait, TraitItem, Type, Visibility};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_paths::Module;

use crate::declared::Declared;
use crate::owner::Owner;

pub struct PairedMethods<'file> {
    file: &'file SourceFile,
    module: Module,
    found: Vec<Declared>,
}

impl PairedMethods<'_> {
    pub fn of_file(file: &SourceFile) -> Vec<Declared> {
        let mut scan = PairedMethods { file, module: Module::of_file(file), found: Vec::new() };
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

    fn owned_by(&mut self, owner: Option<Name>, methods: &[&Ident]) {
        let owner = owner.map(|name| Owner::new(&self.module, name));
        let declared: Vec<Declared> = owner
            .iter()
            .flat_map(|owner| methods.iter().map(move |method| (owner, method)))
            .map(|(owner, method)| {
                let site = Site::new(self.file.path(), LineNumber::from(method.span()));
                Declared::new(owner.clone(), Name::from(*method), site)
            })
            .collect();
        self.found.extend(declared);
    }
}

impl<'ast> Visit<'ast> for PairedMethods<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.module.clone();
        self.module = outer.within(&Name::from(&module.ident));
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.module = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let methods = match (Walk::inherent_impls(block), Walk::outside_tests(&block.attrs)) {
            (Walk::Entered, Walk::Entered) => Self::public_methods(block),
            _ => Vec::new(),
        };
        let owner = match block.self_ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        };
        self.owned_by(owner, &methods);
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        Walk::enter(&block.attrs, || self.owned_by(Some(Name::from(&block.ident)), &Self::declared_methods(block)));
    }
}
