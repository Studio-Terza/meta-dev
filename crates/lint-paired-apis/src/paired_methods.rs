use lint_core::SourceFile;
use std::collections::BTreeMap;
use std::path::Path;

use syntax::{Ident, ItemImpl, ItemMod, ItemTrait, Visibility};

use lint_core::Module;
use lint_core::{FileHead, Parsed, Site, Syntax, Walk};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Contribute, Contributed, Descend, Facts, Kind};

use crate::PairedOwners;
use crate::declared::Declared;
use crate::owner::Owner;

#[derive(Debug)]
pub struct PairedMethods<'file> {
    head: &'file FileHead,
    module: Module<'file>,
    kinds: &'static [Kind],
}

impl<'file> PairedMethods<'file> {
    pub fn of_file(parsed: Parsed<'_, 'file>) -> PairedMethods<'file> {
        let head = parsed.head();
        let kinds = <&SourceFile>::from(parsed)
            .shipped()
            .map_or(&[][..], |_| &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemTrait][..]);
        PairedMethods { head, module: Module::of_head(head), kinds }
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

    fn owned_by(&self, owner: Option<Name>, methods: &[&Ident]) -> Contributed<(), PairedOwners<'file>> {
        let path = AsRef::<Path>::as_ref(self.head);
        let declared: Vec<Declared<'file>> = methods
            .iter()
            .map(|method| Declared::new(Name::from(*method), Site::new(path, LineNumber::from(method.span()))))
            .collect();
        let owners =
            owner.map(|name| (Owner::new(&self.module, name), declared)).into_iter().collect::<BTreeMap<_, _>>();
        Contributed::new((), PairedOwners::new(owners), Descend::Over)
    }
}

impl<'file> Contribute for PairedMethods<'file> {
    type Open = ();
    type Part = PairedOwners<'file>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PairedOwners<'file>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PairedOwners<'file>> {
        let methods = facts
            .inherent()
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Entered)
            .map_or_default(Self::public_methods);
        self.owned_by(facts.name().cloned(), &methods)
    }

    fn on_item_trait(
        &self,
        block: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PairedOwners<'file>> {
        let entered = Descend::from(facts.gate()).entered();
        let declared = entered.map_or_default(|_| Self::declared_methods(block));
        self.owned_by(entered.and_then(|_| facts.name().cloned()), &declared)
    }
}
