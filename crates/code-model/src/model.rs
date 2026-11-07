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
use crate::type_ref::TypeRef;
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
    pub fn declared(&self, item: ItemId) -> Option<&Item> {
        self.declarations.items().get(item.index())
    }

    #[must_use]
    pub fn expanded(&self, ty: &TypeRef) -> TypeRef {
        self.declarations.aliases().expanded(ty)
    }
}
