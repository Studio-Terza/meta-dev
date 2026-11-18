use std::path::PathBuf;

use lint_layout::LineNumber;

use crate::placed::FileHome;
use crate::placed::Placed;
use lint_layout::Site;

#[derive(Debug)]
pub struct FileModel {
    path: PathBuf,
    home: FileHome,
}

#[derive(Debug, Default)]
pub struct FileTable {
    files: Vec<FileModel>,
}

impl FromIterator<FileModel> for FileTable {
    fn from_iter<Files: IntoIterator<Item = FileModel>>(files: Files) -> FileTable {
        FileTable { files: files.into_iter().collect() }
    }
}

impl From<FileTable> for Vec<FileModel> {
    fn from(table: FileTable) -> Vec<FileModel> {
        table.files
    }
}

impl AsRef<[FileModel]> for FileTable {
    fn as_ref(&self) -> &[FileModel] {
        &self.files
    }
}

impl FileModel {
    #[must_use]
    pub fn new(path: PathBuf, home: FileHome) -> FileModel {
        FileModel { path, home }
    }

    pub(crate) fn placed<Entity: Copy>(&self, line: LineNumber, entity: Entity) -> Placed<'_, Entity> {
        Placed::new(Site::new(&self.path, line), &self.home, entity)
    }
}
