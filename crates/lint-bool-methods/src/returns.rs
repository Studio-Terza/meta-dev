use std::path::Path;
mod finding;

use syntax::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Signature, TraitItemFn};

use lint_core::Joins;
use lint_core::{Site, SourceFile, Syntax, Violation, Walk};
use lint_layout::LineNumber;
use lint_sweep::{Concludes, Contribute, Contributed, Descend, Facts, Kind};

use finding::Finding;

#[derive(Debug)]
pub struct Returns<'file> {
    file: &'file SourceFile,
}

type Returned<'file> = Contributed<(), Vec<Violation<'file>>>;

impl<'file> Returns<'file> {
    fn flagged(&self, signature: &Signature) -> Option<Violation<'file>> {
        Syntax::returned(&signature.output)
            .and_then(Syntax::type_path)
            .filter(|path| path.qself.is_none())
            .filter(|path| path.path.is_ident("bool"))
            .map(|_| {
                Violation::new(
                    Site::new(AsRef::<Path>::as_ref(self.file.head()), LineNumber::from(signature.ident.span())),
                    Finding::from(&signature.ident),
                )
            })
    }

    fn opened(&self, attributes: &[Attribute], signature: &Signature) -> Returned<'file> {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let flagged = descend.entered().and_then(|()| self.flagged(signature));
        Contributed::new((), flagged.into_iter().collect(), descend)
    }
}

impl<'file> From<&'file SourceFile> for Returns<'file> {
    fn from(file: &'file SourceFile) -> Returns<'file> {
        Returns { file }
    }
}

impl<'file> Contribute for Returns<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemTrait, Kind::TraitItemFn]
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        self.opened(&function.attrs, &function.sig)
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        Contributed::descending(
            facts.inherent().map_or(Descend::Over, |_| Descend::Into).combine(Descend::from(facts.gate())),
        )
    }

    fn on_impl_item_fn(
        &self,
        method: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        self.opened(&method.attrs, &method.sig)
    }

    fn on_item_trait(
        &self,
        _: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_trait_item_fn(
        &self,
        method: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Returned<'file> {
        self.opened(&method.attrs, &method.sig)
    }
}

impl<'file> Concludes<'file> for Returns<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}
