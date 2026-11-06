use std::path::PathBuf;

use crate::boundary::FileId;
use crate::found::Found;
use crate::implementation::Implementation;
use crate::model::{FileModel, Filed, Model};

#[derive(Debug, Default)]
pub struct Building {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
}

impl Building {
    pub fn file(&mut self, path: PathBuf) -> FileId {
        let file = FileId::next(&self.files);
        self.files.push(FileModel::from(path));
        file
    }

    pub fn implementation(&mut self, file: FileId, found: Found<Implementation>) {
        self.implementations.push(Filed::new(file, found));
    }

    #[must_use]
    pub fn built(mut self) -> Model {
        self.implementations.sort_by_key(Filed::file);
        Model::new(self.files, self.implementations)
    }
}
