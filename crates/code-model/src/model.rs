mod building;
mod declarations;
mod file_model;
mod filed;

use crate::boundary::ItemId;
use crate::call::Call;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::Item;
use crate::placed::Placed;
use crate::region::Region;
use crate::sited::Sited;
use crate::type_ref::{Named, Steps, TypeRef};
pub use building::Building;
pub(crate) use declarations::{Aliases, Declarations};
pub(crate) use file_model::FileModel;
pub(crate) use filed::Filed;

#[derive(Debug, Default)]
pub struct Model {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
    functions: Vec<Filed<Function>>,
    holdings: Vec<Filed<Holding>>,
    calls: Vec<Filed<Call>>,
    declarations: Declarations,
}

#[derive(Clone, Copy, Debug)]
enum Admitted {
    Definite,
    Every,
}

impl Admitted {
    fn admits<'model>(self, placed: Placed<'model, &'model Function>) -> Option<Placed<'model, &'model Function>> {
        match self {
            Admitted::Definite => Model::definite(placed),
            Admitted::Every => Some(placed),
        }
    }
}

#[derive(Debug)]
pub struct Functions<'model> {
    files: &'model [FileModel],
    rows: std::slice::Iter<'model, Filed<Function>>,
    admitted: Admitted,
}

impl<'model> Functions<'model> {
    #[must_use]
    pub fn everywhere(self) -> Functions<'model> {
        Functions { files: self.files, rows: self.rows, admitted: Admitted::Every }
    }
}

impl<'model> Iterator for Functions<'model> {
    type Item = Placed<'model, &'model Function>;

    fn next(&mut self) -> Option<Placed<'model, &'model Function>> {
        let files = self.files;
        let admitted = self.admitted;
        self.rows.find_map(|filed| filed.placed(files).and_then(|placed| admitted.admits(placed)))
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

impl Model {
    pub fn implementations(&self) -> impl Iterator<Item = Placed<'_, &Implementation>> {
        self.every_implementation().filter_map(Model::definite)
    }

    #[must_use]
    pub fn every_implementation(&self) -> Implementations<'_> {
        Implementations { files: &self.files, rows: self.implementations.iter() }
    }

    #[must_use]
    pub fn functions(&self) -> Functions<'_> {
        Functions { files: &self.files, rows: self.functions.iter(), admitted: Admitted::Definite }
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
    pub fn declared(&self, item: ItemId) -> Option<&Item> {
        self.declarations.items().get(item.index())
    }

    #[must_use]
    pub fn expanded(&self, ty: &TypeRef) -> TypeRef {
        ty.expanded(self.declarations.aliases(), Steps::LIMIT).unwrap_or_else(|| ty.clone())
    }

    #[must_use]
    pub fn aliased(&self, named: &Named<Sited>) -> Option<TypeRef> {
        named.expansion(self.declarations.aliases())
    }

    fn definite<'model, Entity: AsRef<Region>>(
        placed: Placed<'model, &'model Entity>,
    ) -> Option<Placed<'model, &'model Entity>> {
        placed.entity().as_ref().reach().definite().map(|_| placed)
    }
}
