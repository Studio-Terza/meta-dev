use syn::visit::{self, Visit};
use syn::{ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};

use crate::declaration::Declaration;

pub(crate) struct Declared<'file> {
    file: &'file SourceFile,
    found: Vec<Declaration<'file>>,
}

impl<'file> Declared<'file> {
    const DISPATCHING: &'static str = "enum_dispatch";

    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Declaration<'file>> {
        let mut scan = Declared { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn methods(block: &ItemTrait) -> impl Iterator<Item = &TraitItemFn> {
        block.items.iter().filter_map(Self::method)
    }

    const fn method(item: &TraitItem) -> Option<&TraitItemFn> {
        match item {
            TraitItem::Fn(method) => Some(method),
            _ => None,
        }
    }

    fn with_methods(block: &ItemTrait) -> Option<&ItemTrait> {
        match block.attrs.iter().find(|attribute| attribute.path().is_ident(Self::DISPATCHING)) {
            Some(_) => None,
            None => Self::methods(block).next().map(|_| block),
        }
    }

    fn provided(block: &ItemTrait) -> Vec<Name> {
        Self::methods(block)
            .filter(|method| method.default.is_some())
            .map(|method| Name::from(&method.sig.ident))
            .collect()
    }
}

impl<'ast> Visit<'ast> for Declared<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        let site = Site::new(self.file.path(), LineNumber::from(block.ident.span()));
        let declaration = Declaration::new(Name::from(&block.ident), Self::provided(block), site);
        self.found.extend(
            Self::with_methods(block)
                .filter(|block| Walk::outside_tests(&block.attrs) == Walk::Entered)
                .map(|_| declaration),
        );
    }
}
