mod file_model;
mod filed;

use crate::implementation::Implementation;
use crate::placed::Placed;
pub(crate) use file_model::FileModel;
pub(crate) use filed::Filed;

#[derive(Debug, Default)]
pub struct Model {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
}

impl Model {
    pub(crate) fn new(files: Vec<FileModel>, implementations: Vec<Filed<Implementation>>) -> Model {
        Model { files, implementations }
    }

    pub fn implementations(&self) -> impl Iterator<Item = Placed<'_, &Implementation>> {
        self.implementations.iter().filter_map(|filed| filed.placed(&self.files))
    }
}
