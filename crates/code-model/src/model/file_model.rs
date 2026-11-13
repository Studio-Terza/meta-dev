use std::path::PathBuf;

use lint_layout::{LineNumber, Name};

use crate::placed::Placed;
use crate::site::Site;

#[derive(Debug)]
pub(crate) struct FileModel {
    path: PathBuf,
    krate: Name,
    root: PathBuf,
}

impl FileModel {
    pub(crate) fn new(path: PathBuf, krate: Name, root: PathBuf) -> FileModel {
        FileModel { path, krate, root }
    }

    pub(crate) fn placed<Entity: Copy>(&self, line: LineNumber, entity: Entity) -> Placed<'_, Entity> {
        Placed::new(Site::new(&self.path, line), &self.krate, &self.root, entity)
    }
}
