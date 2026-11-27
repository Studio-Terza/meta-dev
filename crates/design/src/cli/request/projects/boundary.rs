use std::path::PathBuf;

use super::Located;

impl Located {
    pub(crate) fn spec(&self, project: &str) -> PathBuf {
        self.specs.join(project).with_extension(Located::SPEC)
    }
}
