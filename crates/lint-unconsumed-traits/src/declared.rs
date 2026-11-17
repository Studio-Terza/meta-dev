use std::path::Path;
mod declaration;
mod finding;

use syntax::{ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use crate::Consumers;
use finding::Finding;
use lint_layout::{LineNumber, Name};

use declaration::Declaration;

#[derive(Debug)]
pub struct Declared<'file> {
    path: &'file std::path::Path,
}

#[derive(Debug, Default)]
pub struct Declarations<'file>(Vec<Declaration<'file>>);

impl<'file> Declarations<'file> {
    pub(crate) fn unconsumed(self, consumers: &Consumers) -> Vec<Violation<'file>> {
        self.0
            .into_iter()
            .filter_map(|declaration| declaration.unconsumed(consumers))
            .map(|declaration| Violation::new(*AsRef::<Site>::as_ref(&declaration), Finding::from(&declaration)))
            .collect()
    }
}

impl Declared<'_> {
    const DISPATCHING: &'static str = "enum_dispatch";

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

impl<'file> Fold for Declared<'file> {
    type Found = Declarations<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemTrait]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_trait(&self, found: Self::Found, block: &ItemTrait, facts: &Facts<'_, ItemTrait>) -> Visit<Self::Found> {
        let site = Site::new(self.path, LineNumber::from(block.ident.span()));
        let declaration = facts.name().map(|name| Declaration::new(name.clone(), Self::provided(block), site));
        let declared = Self::with_methods(block)
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Entered)
            .and(declaration);
        Visit::new(Declarations(found.0.into_iter().chain(declared).collect()), Descend::Over)
    }
}

impl<'file> From<&'file SourceFile> for Declared<'file> {
    fn from(file: &'file SourceFile) -> Declared<'file> {
        Declared { path: AsRef::<Path>::as_ref(file.head()) }
    }
}
