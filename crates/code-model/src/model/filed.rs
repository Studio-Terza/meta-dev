use crate::boundary::FileId;
use crate::found::Found;
use crate::placed::Placed;
use crate::site::Site;

use super::file_model::FileModel;

#[derive(Debug)]
pub(crate) struct Filed<Entity> {
    file: FileId,
    found: Found<Entity>,
}

impl<Entity> Filed<Entity> {
    pub(crate) const fn new(file: FileId, found: Found<Entity>) -> Self {
        Self { file, found }
    }

    pub(crate) fn sorted(rows: Vec<Self>) -> Vec<Self> {
        let mut sorted = rows;
        sorted.sort_by_key(|filed| filed.file);
        sorted
    }

    pub(crate) fn placed<'model>(&'model self, files: &'model [FileModel]) -> Option<Placed<'model, &'model Entity>> {
        let file = self.file.within(files)?;
        Some(Placed::new(Site::new(file.path(), self.found.line()), self.found.entity()))
    }
}
