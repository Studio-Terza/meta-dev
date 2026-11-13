mod declaration;
mod finding;

use syn::{ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_sweep::{Descend, Finds, Kind, Watch};

use crate::Consumers;
use finding::Finding;
use lint_layout::{LineNumber, Name};

use declaration::Declaration;

#[derive(Debug)]
pub(crate) struct Declared<'file, 'run> {
    path: &'file std::path::Path,
    consumers: &'run Consumers,
    found: Vec<Declaration<'file>>,
}

impl<'file, 'run> Declared<'file, 'run> {
    const DISPATCHING: &'static str = "enum_dispatch";

    pub(crate) fn new(file: &'file SourceFile, consumers: &'run Consumers) -> Declared<'file, 'run> {
        Declared { path: file.head().path(), consumers, found: Vec::new() }
    }

    fn methods(block: &ItemTrait) -> impl Iterator<Item = &TraitItemFn> {
        block.items.iter().filter_map(Self::method)
    }

    fn method(item: &TraitItem) -> Option<&TraitItemFn> {
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

impl<'ast> Watch<'ast> for Declared<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemTrait]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_trait(&mut self, block: &'ast ItemTrait) -> Descend {
        let site = Site::new(self.path, LineNumber::from(block.ident.span()));
        let declaration = Declaration::new(Name::from(&block.ident), Self::provided(block), site);
        self.found.extend(
            Self::with_methods(block)
                .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Entered)
                .map(|_| declaration),
        );
        Descend::Over
    }
}

impl<'file> Finds<'file> for Declared<'file, '_> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let consumers = self.consumers;
        self.found
            .into_iter()
            .filter_map(|declaration| declaration.unconsumed(consumers))
            .map(|declaration| Violation::new(*AsRef::<Site>::as_ref(&declaration), Finding::from(&declaration)))
            .collect()
    }
}
