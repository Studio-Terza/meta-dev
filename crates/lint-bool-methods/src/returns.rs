mod finding;

use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Signature, TraitItemFn};

use lint_core::{Site, SourceFile, Syntax, Violation, Walk};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Finds, Kind, Watch};

use finding::Finding;

#[derive(Debug)]
pub struct Returns<'file> {
    file: &'file SourceFile,
    found: Vec<Violation<'file>>,
}

impl<'file> Returns<'file> {
    fn flagged(&self, signature: &Signature) -> Option<Violation<'file>> {
        Syntax::returned(&signature.output)
            .and_then(Syntax::type_path)
            .filter(|path| path.qself.is_none())
            .filter(|path| path.path.is_ident("bool"))
            .map(|_| {
                Violation::new(
                    Site::new(self.file.head().path(), LineNumber::from(signature.ident.span())),
                    Finding::from(&signature.ident),
                )
            })
    }

    fn opened(&mut self, attributes: &[Attribute], signature: &Signature) -> Descend {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let flagged = descend.entered().and_then(|_| self.flagged(signature));
        self.found.extend(flagged);
        descend
    }
}

impl<'file> From<&'file SourceFile> for Returns<'file> {
    fn from(file: &'file SourceFile) -> Returns<'file> {
        Returns { file, found: Vec::new() }
    }
}

impl<'ast> Watch<'ast> for Returns<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemTrait, Kind::TraitItemFn]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.opened(&function.attrs, &function.sig)
    }

    fn on_item_impl(&mut self, block: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::inherent_impls(block)).then(|| Descend::from(Walk::outside_tests(&block.attrs)))
    }

    fn on_impl_item_fn(&mut self, method: &'ast ImplItemFn) -> Descend {
        self.opened(&method.attrs, &method.sig)
    }

    fn on_item_trait(&mut self, block: &'ast ItemTrait) -> Descend {
        Descend::from(Walk::outside_tests(&block.attrs))
    }

    fn on_trait_item_fn(&mut self, method: &'ast TraitItemFn) -> Descend {
        self.opened(&method.attrs, &method.sig)
    }
}

impl<'file> Finds<'file> for Returns<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.found
    }
}
