mod derived;
mod finding;

use std::path::Path;

use syntax::{ItemStruct, Lifetime, TypeReference};

use algebra::{Monoid, Semigroup};
use derived::Derived;
use finding::Finding;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Concludes, Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Holding {
    #[default]
    Owning,
    Borrowing,
}

impl Semigroup for Holding {
    fn combine(self, other: Holding) -> Holding {
        match self {
            Holding::Owning => other,
            Holding::Borrowing => Holding::Borrowing,
        }
    }
}

impl Monoid for Holding {}

#[derive(Debug)]
pub struct Ungenerated<'file> {
    path: &'file Path,
}

impl<'file> Contribute for Ungenerated<'file> {
    type Open = Holding;
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemStruct, Kind::TypeReference, Kind::Lifetime]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemStruct)] }
    }

    fn off_item_struct(
        &self,
        item: &ItemStruct,
        facts: &Facts<'_, ItemStruct>,
        inner: Contribution<Holding, Vec<Violation<'file>>>,
    ) -> Vec<Violation<'file>> {
        inner.concluded(|holding, found| {
            let derived = Derived::from(item.attrs.as_slice());
            let flagged =
                derived.lacks_generator().zip(facts.name()).filter(|_| holding == Holding::Owning).map(|(_, name)| {
                    Violation::new(
                        Site::new(self.path, LineNumber::from(item.struct_token.span)),
                        Finding::from(name.clone()),
                    )
                });
            found.into_iter().chain(flagged).collect()
        })
    }

    fn on_type_reference(
        &self,
        _: &TypeReference,
        _: &Facts<'_, TypeReference>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Holding, Vec<Violation<'file>>> {
        Contributed::new(Holding::Borrowing, Vec::new(), Descend::Into)
    }

    fn on_lifetime(
        &self,
        _: &Lifetime,
        _: &Facts<'_, Lifetime>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Holding, Vec<Violation<'file>>> {
        Contributed::new(Holding::Borrowing, Vec::new(), Descend::Into)
    }
}

impl<'file> Concludes<'file> for Ungenerated<'file> {
    fn violations(self, _: Holding, found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}

impl<'file> From<&'file SourceFile> for Ungenerated<'file> {
    fn from(file: &'file SourceFile) -> Ungenerated<'file> {
        Ungenerated { path: AsRef::<Path>::as_ref(file.head()) }
    }
}
