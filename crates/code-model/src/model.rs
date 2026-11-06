mod boundary;
mod file_model;

use std::path::PathBuf;

use crate::found::Found;
use crate::implementation::Implementation;
use crate::placed::Placed;
use crate::site::Site;
use boundary::FileId;
use file_model::FileModel;

#[derive(Debug, Default)]
pub struct Model {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
}

#[derive(Debug)]
struct Filed<Entity> {
    file: FileId,
    found: Found<Entity>,
}

impl Model {
    #[must_use]
    pub fn with_file(mut self, path: PathBuf, found: Vec<Found<Implementation>>) -> Model {
        let file = FileId::next(&self.files);
        self.files.push(FileModel::from(path));
        self.implementations.extend(found.into_iter().map(|found| Filed { file, found }));
        self
    }

    pub fn implementations(&self) -> impl Iterator<Item = Placed<'_, &Implementation>> {
        self.implementations.iter().filter_map(|filed| {
            let file = filed.file.within(&self.files)?;
            Some(Placed::new(Site::new(file.path(), filed.found.line()), filed.found.entity()))
        })
    }
}
