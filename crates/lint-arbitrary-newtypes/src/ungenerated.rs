mod derived;
mod finding;

use std::path::Path;

use syn::{ItemStruct, Lifetime, TypeReference};

use derived::Derived;
use finding::Finding;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Descend, Finds, Kind, Watch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Holding {
    Owning,
    Borrowing,
}

#[derive(Debug)]
struct Open<'file> {
    flagged: Option<Violation<'file>>,
    holding: Holding,
}

#[derive(Debug)]
pub struct Ungenerated<'file> {
    path: &'file Path,
    open: Vec<Open<'file>>,
    found: Vec<Violation<'file>>,
}

impl Open<'_> {
    fn borrow(&mut self) {
        self.holding = Holding::Borrowing;
    }
}

impl Ungenerated<'_> {
    fn borrowed(&mut self) -> Descend {
        self.open.last_mut().into_iter().for_each(Open::borrow);
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Ungenerated<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemStruct, Kind::TypeReference, Kind::Lifetime]
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        let derived = Derived::from(item.attrs.as_slice());
        let flagged = derived.lacks_generator().map(|_| {
            Violation::new(
                Site::new(self.path, LineNumber::from(item.struct_token.span)),
                Finding::from(Name::from(&item.ident)),
            )
        });
        self.open.push(Open { flagged, holding: Holding::Owning });
        Descend::Into
    }

    fn off_item_struct(&mut self, _: &'ast ItemStruct) {
        let closed = self.open.pop().filter(|open| open.holding == Holding::Owning);
        self.found.extend(closed.and_then(|open| open.flagged));
    }

    fn on_type_reference(&mut self, _: &'ast TypeReference) -> Descend {
        self.borrowed()
    }

    fn on_lifetime(&mut self, _: &'ast Lifetime) -> Descend {
        self.borrowed()
    }
}

impl<'file> From<&'file SourceFile> for Ungenerated<'file> {
    fn from(file: &'file SourceFile) -> Ungenerated<'file> {
        Ungenerated { path: file.head().path(), open: Vec::new(), found: Vec::new() }
    }
}

impl<'file> Finds<'file> for Ungenerated<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.found
    }
}
