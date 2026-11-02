use syn::visit::{self, Visit};
use syn::{ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};

use crate::declaration::Declaration;

pub(crate) struct Declared<'file> {
    file: &'file SourceFile,
    found: Vec<Declaration>,
}

impl Declared<'_> {
    const DISPATCHING: &'static str = "enum_dispatch";

    pub(crate) fn of_file(file: &SourceFile) -> Vec<Declaration> {
        let mut scan = Declared { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn methods(block: &ItemTrait) -> impl Iterator<Item = &TraitItemFn> {
        block.items.iter().filter_map(|item| match item {
            TraitItem::Fn(method) => Some(method),
            _ => None,
        })
    }

    fn is_bare(block: &ItemTrait) -> bool {
        Self::methods(block).next().is_some()
            && Self::methods(block).all(|method| method.default.is_none())
            && !block.attrs.iter().any(|attribute| attribute.path().is_ident(Self::DISPATCHING))
    }
}

impl<'ast> Visit<'ast> for Declared<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_trait(&mut self, block: &'ast ItemTrait) {
        let site = Site::new(self.file.path(), LineNumber::from(block.ident.span()));
        let declaration = Declaration::new(Name::from(&block.ident), site);
        self.found.extend(
            (Walk::outside_tests(&block.attrs) == Walk::Entered && Self::is_bare(block)).then_some(declaration),
        );
    }
}
