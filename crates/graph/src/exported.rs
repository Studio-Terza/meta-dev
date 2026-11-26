use crate::item_path::ItemPath;
use crate::resolved::Resolved;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exported<'model> {
    path: &'model ItemPath,
    resolved: &'model Resolved,
}

impl<'model> Exported<'model> {
    #[must_use]
    pub fn new(path: &'model ItemPath, resolved: &'model Resolved) -> Exported<'model> {
        Exported { path, resolved }
    }

    #[must_use]
    pub fn path(self) -> &'model ItemPath {
        self.path
    }

    #[must_use]
    pub fn resolved(self) -> &'model Resolved {
        self.resolved
    }
}
