use std::path::Path;
mod finding;

use syntax::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Signature, TraitItemFn};

use lint_core::Extended;
use lint_core::{Site, SourceFile, Syntax, Violation, Walk};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use finding::Finding;

#[derive(Debug)]
pub struct Returns<'file> {
    file: &'file SourceFile,
}

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

    fn opened(
        &self,
        found: Vec<Violation<'file>>,
        attributes: &[Attribute],
        signature: &Signature,
    ) -> Visit<Vec<Violation<'file>>> {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let flagged = descend.entered().and_then(|_| self.flagged(signature));
        Visit::new(found.extended(flagged), descend)
    }
}

impl<'file> From<&'file SourceFile> for Returns<'file> {
    fn from(file: &'file SourceFile) -> Returns<'file> {
        Returns { file }
    }
}

impl<'file> Fold for Returns<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemTrait, Kind::TraitItemFn]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_fn(&self, found: Self::Found, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Self::Found> {
        self.opened(found, &function.attrs, &function.sig)
    }

    fn on_item_impl(&self, found: Self::Found, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        Visit::new(
            found,
            facts.inherent().map_or(Descend::Over, |_| Descend::Into).then(|| Descend::from(facts.gate())),
        )
    }

    fn on_impl_item_fn(
        &self,
        found: Self::Found,
        method: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Self::Found> {
        self.opened(found, &method.attrs, &method.sig)
    }

    fn on_item_trait(&self, found: Self::Found, _: &ItemTrait, facts: &Facts<'_, ItemTrait>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_trait_item_fn(
        &self,
        found: Self::Found,
        method: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Self::Found> {
        self.opened(found, &method.attrs, &method.sig)
    }
}
