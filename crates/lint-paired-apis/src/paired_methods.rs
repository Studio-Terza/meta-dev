use std::collections::BTreeMap;

use syn::{Ident, ItemImpl, ItemMod, ItemTrait, Visibility};

use lint_core::Module;
use lint_core::{FileHead, Parsed, Site, Syntax, Walk};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};

use crate::declared::Declared;
use crate::owner::Owner;

#[derive(Debug)]
pub struct PairedMethods<'file> {
    head: &'file FileHead,
    module: Module<'file>,
    found: BTreeMap<Owner<'file>, Vec<Declared<'file>>>,
    kinds: &'static [Kind],
}

impl<'file> PairedMethods<'file> {
    pub fn of_file(parsed: Parsed<'_, 'file>) -> PairedMethods<'file> {
        let head = parsed.head();
        let kinds = parsed.file().shipped().map_or(&[][..], |_| &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemTrait][..]);
        PairedMethods { head, module: Module::of_head(head), found: BTreeMap::new(), kinds }
    }

    fn public_methods(block: &ItemImpl) -> Vec<&Ident> {
        block
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .filter(|method| matches!(method.vis, Visibility::Public(_)))
            .map(|method| &method.sig.ident)
            .collect()
    }

    fn declared_methods(block: &ItemTrait) -> Vec<&Ident> {
        block.items.iter().filter_map(Syntax::trait_fn).map(|method| &method.sig.ident).collect()
    }

    fn owned_by(&mut self, owner: Option<Name>, methods: &[&Ident]) {
        let path = self.head.path();
        let declared = methods
            .iter()
            .map(|method| Declared::new(Name::from(*method), Site::new(path, LineNumber::from(method.span()))));
        owner.zip(Some(declared)).into_iter().for_each(|(name, listed)| {
            self.found.entry(Owner::new(&self.module, name)).or_default().extend(listed);
        });
    }
}

impl<'ast> Watch<'ast> for PairedMethods<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, block: &'ast ItemImpl) -> Descend {
        let methods = Syntax::inherent(block)
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Entered)
            .map_or_default(Self::public_methods);
        self.owned_by(Name::of_type(&block.self_ty), &methods);
        Descend::Over
    }

    fn on_item_trait(&mut self, block: &'ast ItemTrait) -> Descend {
        let entered = Descend::from(Walk::outside_tests(&block.attrs)).entered();
        let declared = entered.map_or_default(|_| Self::declared_methods(block));
        self.owned_by(entered.map(|_| Name::from(&block.ident)), &declared);
        Descend::Over
    }
}

impl<'file> Gathers<BTreeMap<Owner<'file>, Vec<Declared<'file>>>> for PairedMethods<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> BTreeMap<Owner<'file>, Vec<Declared<'file>>> {
        self.found
    }
}
