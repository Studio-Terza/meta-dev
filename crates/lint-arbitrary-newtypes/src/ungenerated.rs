mod derived;
mod finding;

use std::path::Path;

use syntax::{ItemStruct, Lifetime, TypeReference};

use derived::Derived;
use finding::Finding;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Facts, Fold, Kind, Visit};

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
}

#[derive(Debug, Default)]
pub struct Structs<'file> {
    open: Vec<Open<'file>>,
    found: Vec<Violation<'file>>,
}

impl<'file> Structs<'file> {
    fn borrowed(self) -> Visit<Structs<'file>> {
        let mut open = self.open;
        let last = open.pop().map(|top| Open { holding: Holding::Borrowing, ..top });
        Visit::descending(Structs { open: open.into_iter().chain(last).collect(), ..self })
    }

    fn opened(self, flagged: Option<Violation<'file>>) -> Structs<'file> {
        Structs { open: self.open.into_iter().chain([Open { flagged, holding: Holding::Owning }]).collect(), ..self }
    }

    fn closed(self) -> Structs<'file> {
        let mut open = self.open;
        let owning = open.pop().filter(|last| last.holding == Holding::Owning);
        Structs { open, found: self.found.into_iter().chain(owning.and_then(|last| last.flagged)).collect() }
    }
}

impl<'file> From<Structs<'file>> for Vec<Violation<'file>> {
    fn from(structs: Structs<'file>) -> Vec<Violation<'file>> {
        structs.found
    }
}

impl<'file> Fold for Ungenerated<'file> {
    type Found = Structs<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemStruct, Kind::TypeReference, Kind::Lifetime]
    }

    fn on_item_struct(
        &self,
        found: Structs<'file>,
        item: &ItemStruct,
        facts: &Facts<'_, ItemStruct>,
    ) -> Visit<Structs<'file>> {
        let derived = Derived::from(item.attrs.as_slice());
        let flagged = derived.lacks_generator().zip(facts.name()).map(|(_, name)| {
            Violation::new(Site::new(self.path, LineNumber::from(item.struct_token.span)), Finding::from(name.clone()))
        });
        Visit::descending(found.opened(flagged))
    }

    fn off_item_struct(&self, found: Structs<'file>, _: &ItemStruct, _: &Facts<'_, ItemStruct>) -> Structs<'file> {
        found.closed()
    }

    fn on_type_reference(
        &self,
        found: Structs<'file>,
        _: &TypeReference,
        _: &Facts<'_, TypeReference>,
    ) -> Visit<Structs<'file>> {
        found.borrowed()
    }

    fn on_lifetime(&self, found: Structs<'file>, _: &Lifetime, _: &Facts<'_, Lifetime>) -> Visit<Structs<'file>> {
        found.borrowed()
    }
}

impl<'file> From<&'file SourceFile> for Ungenerated<'file> {
    fn from(file: &'file SourceFile) -> Ungenerated<'file> {
        Ungenerated { path: AsRef::<Path>::as_ref(file.head()) }
    }
}
