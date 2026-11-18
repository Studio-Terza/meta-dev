use std::path::Path;
mod declaration;
mod finding;

use algebra::{Monoid, Semigroup};
use syntax::{ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Watching};

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

impl Semigroup for Declarations<'_> {
    fn combine(self, other: Self) -> Self {
        Declarations(self.0.combine(other.0))
    }
}

impl Monoid for Declarations<'_> {}

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

impl<'file> Contribute for Declared<'file> {
    type Open = ();
    type Part = Declarations<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemTrait]
    }

    fn on_item_mod(&self, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Contributed<(), Declarations<'file>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_trait(&self, block: &ItemTrait, facts: &Facts<'_, ItemTrait>) -> Contributed<(), Declarations<'file>> {
        let site = Site::new(self.path, LineNumber::from(block.ident.span()));
        let declaration = facts.name().map(|name| Declaration::new(name.clone(), Self::provided(block), site));
        let found = Self::with_methods(block)
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Entered)
            .and(declaration);
        Contributed::new((), Declarations(found.into_iter().collect()), Descend::Over)
    }
}

impl<'file> Gathers<Declarations<'file>> for Contributing<Declared<'file>> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(self)
    }

    fn gathered(self) -> Declarations<'file> {
        self.concluded(|_, (), found| found)
    }
}

impl<'file> From<&'file SourceFile> for Declared<'file> {
    fn from(file: &'file SourceFile) -> Declared<'file> {
        Declared { path: AsRef::<Path>::as_ref(file.head()) }
    }
}
