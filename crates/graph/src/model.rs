mod declarations;
mod file_model;
mod filed;
mod part;

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::OnceLock;

use algebra::Monoid;

use crate::boundary::ItemId;
use crate::call::Call;
use crate::crate_root::CrateRoots;
use crate::dispatch::Dispatch;
use crate::domain::Domain;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::Item;
use crate::item_declaration::ItemDeclaration;
use crate::item_path::ItemPath;
use crate::law::Law;
use crate::path_use::PathUse;
use crate::placed::Placed;
use crate::quoted::Quoted;
use crate::references::{Consumed, Consumers, References};
use crate::resolved::Resolved;
use crate::type_ref::TypeRef;
pub use declarations::{Aliases, Declarations, Exports, Issued};
pub use file_model::{FileModel, FileTable};
pub(crate) use filed::Filed;
pub use part::ModelPart;

#[derive(Debug, Default)]
pub struct Model {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
    functions: Vec<Filed<Function>>,
    holdings: Vec<Filed<Holding>>,
    calls: Vec<Filed<Call>>,
    uses: Vec<Filed<PathUse>>,
    interpolated: Vec<Filed<PathUse>>,
    typed: Vec<Filed<PathUse>>,
    dispatched: Vec<Dispatch>,
    quoted: Vec<Filed<Quoted>>,
    placed: Vec<Filed<ItemDeclaration>>,
    laws: Vec<Filed<Law>>,
    declarations: Declarations,
    roots: CrateRoots,
    references: OnceLock<References>,
}

#[derive(Debug)]
pub struct Functions<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Function>>,
}

impl<'model> Iterator for Functions<'model> {
    type Item = Placed<'model, &'model Function>;

    fn next(&mut self) -> Option<Placed<'model, &'model Function>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct Implementations<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Implementation>>,
}

impl<'model> Iterator for Implementations<'model> {
    type Item = Placed<'model, &'model Implementation>;

    fn next(&mut self) -> Option<Placed<'model, &'model Implementation>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct Holdings<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Holding>>,
}

impl<'model> Iterator for Holdings<'model> {
    type Item = Placed<'model, &'model Holding>;

    fn next(&mut self) -> Option<Placed<'model, &'model Holding>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct Calls<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Call>>,
}

impl<'model> Iterator for Calls<'model> {
    type Item = Placed<'model, &'model Call>;

    fn next(&mut self) -> Option<Placed<'model, &'model Call>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct PlacedItems<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<ItemDeclaration>>,
}

impl<'model> Iterator for PlacedItems<'model> {
    type Item = Placed<'model, &'model ItemDeclaration>;

    fn next(&mut self) -> Option<Placed<'model, &'model ItemDeclaration>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct Laws<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Law>>,
}

impl<'model> Iterator for Laws<'model> {
    type Item = Placed<'model, &'model Law>;

    fn next(&mut self) -> Option<Placed<'model, &'model Law>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct Quotations<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Quoted>>,
}

impl<'model> Iterator for Quotations<'model> {
    type Item = Placed<'model, &'model Quoted>;

    fn next(&mut self) -> Option<Placed<'model, &'model Quoted>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

#[derive(Debug)]
pub struct PathUses<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<PathUse>>,
}

impl<'model> Iterator for PathUses<'model> {
    type Item = Placed<'model, &'model PathUse>;

    fn next(&mut self) -> Option<Placed<'model, &'model PathUse>> {
        let files = self.files;
        self.rows.find_map(|filed| filed.placed(files))
    }
}

impl Model {
    #[must_use]
    pub fn implementations(&self) -> Implementations<'_> {
        Implementations { files: &self.files, rows: self.implementations.iter() }
    }

    #[must_use]
    pub fn functions(&self) -> Functions<'_> {
        Functions { files: &self.files, rows: self.functions.iter() }
    }

    #[must_use]
    pub fn holdings(&self) -> Holdings<'_> {
        Holdings { files: &self.files, rows: self.holdings.iter() }
    }

    #[must_use]
    pub fn calls(&self) -> Calls<'_> {
        Calls { files: &self.files, rows: self.calls.iter() }
    }

    #[must_use]
    pub fn uses(&self) -> PathUses<'_> {
        PathUses { files: &self.files, rows: self.uses.iter() }
    }

    pub fn references(&self) -> &References {
        self.references.get_or_init(|| {
            References::concat(
                self.holdings()
                    .map(|placed| References::homed(self, placed))
                    .chain(self.implementations().map(|placed| References::homed(self, placed)))
                    .chain(self.functions().map(|placed| References::homed(self, placed)))
                    .chain(self.mentions().map(|placed| References::homed(self, placed))),
            )
        })
    }

    #[must_use]
    pub fn consumers(&self) -> Consumers<'_> {
        self.holdings()
            .map(Consumers::from)
            .chain(self.implementations().map(Consumers::from))
            .chain(self.functions().map(Consumers::from))
            .chain(self.mentions().map(Consumers::from))
            .collect()
    }

    #[must_use]
    pub fn consumed_by(&self, files: &BTreeSet<&Path>) -> BTreeSet<&Path> {
        let consumed: Consumed = [
            Consumed::within(files, self.holdings()),
            Consumed::within(files, self.implementations()),
            Consumed::within(files, self.functions()),
            Consumed::within(files, self.mentions()),
        ]
        .into_iter()
        .collect();
        consumed.roots(self.items())
    }

    #[must_use]
    pub fn dispatches(&self) -> &[Dispatch] {
        &self.dispatched
    }

    pub fn mentions(&self) -> impl Iterator<Item = Placed<'_, &PathUse>> {
        self.uses()
            .chain(PathUses { files: &self.files, rows: self.interpolated.iter() })
            .chain(PathUses { files: &self.files, rows: self.typed.iter() })
    }

    #[must_use]
    pub fn quotations(&self) -> Quotations<'_> {
        Quotations { files: &self.files, rows: self.quoted.iter() }
    }

    #[must_use]
    pub fn laws(&self) -> Laws<'_> {
        Laws { files: &self.files, rows: self.laws.iter() }
    }

    #[must_use]
    pub fn items(&self) -> PlacedItems<'_> {
        PlacedItems { files: &self.files, rows: self.placed.iter() }
    }

    #[must_use]
    pub fn declared(&self, item: ItemId) -> Option<&Item> {
        self.declarations.items().get(item.index())
    }

    #[must_use]
    pub fn exported(&self, domain: Domain, path: &ItemPath) -> Option<&Resolved> {
        self.declarations.exports().get(&domain).and_then(|paths| paths.get(path))
    }

    #[must_use]
    pub fn with_roots(self, roots: CrateRoots) -> Model {
        Model { roots, ..self }
    }

    #[must_use]
    pub fn roots(&self) -> &CrateRoots {
        &self.roots
    }

    #[must_use]
    pub fn expanded(&self, ty: &TypeRef) -> TypeRef {
        self.declarations.aliases().expanded(ty)
    }
}
