use std::path::PathBuf;

use lint_layout::LineNumber;

use crate::placed::FileHome;
use crate::placed::Placed;
use crate::site::Site;

#[derive(Debug)]
pub(crate) struct FileModel {
    path: PathBuf,
    home: FileHome,
}

impl FileModel {
    pub(crate) fn new(path: PathBuf, home: FileHome) -> FileModel {
        FileModel { path, home }
    }

    pub(crate) fn placed<Entity: Copy>(&self, line: LineNumber, entity: Entity) -> Placed<'_, Entity> {
        Placed::new(Site::new(&self.path, line), &self.home, entity)
    }
}
