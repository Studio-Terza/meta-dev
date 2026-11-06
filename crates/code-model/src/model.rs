mod declarations;
mod file_model;
mod filed;

use crate::boundary::ItemId;
use crate::function::Function;
use crate::implementation::Implementation;
use crate::item::Item;
use crate::placed::Placed;
use crate::type_ref::{Steps, TypeRef};
pub(crate) use declarations::{Aliases, Declarations};
pub(crate) use file_model::FileModel;
pub(crate) use filed::Filed;

#[derive(Debug, Default)]
pub struct Model {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
    functions: Vec<Filed<Function>>,
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

impl Model {
    pub(crate) fn new(
        files: Vec<FileModel>,
        implementations: Vec<Filed<Implementation>>,
        functions: Vec<Filed<Function>>,
        declarations: Declarations,
    ) -> Model {
        Model { files, implementations, functions, declarations }
    }

    pub fn implementations(&self) -> impl Iterator<Item = Placed<'_, &Implementation>> {
        self.implementations.iter().filter_map(|filed| filed.placed(&self.files))
    }

    #[must_use]
    pub fn functions(&self) -> Functions<'_> {
        Functions { files: &self.files, rows: self.functions.iter() }
    }

    #[must_use]
    pub fn declared(&self, item: ItemId) -> Option<&Item> {
        self.declarations.items().get(item.index())
    }

    #[must_use]
    pub fn expanded(&self, ty: &TypeRef) -> TypeRef {
        ty.expanded(self.declarations.aliases(), Steps::LIMIT).unwrap_or_else(|| ty.clone())
    }
}
