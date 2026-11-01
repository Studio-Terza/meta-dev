use std::path::{Path, PathBuf};

use super::tree::Tree;

#[derive(Debug)]
pub struct Scope<'tree> {
    tree: &'tree Tree,
    package: PathBuf,
}

impl<'tree> Scope<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree, package: &Path) -> Scope<'tree> {
        Scope { tree, package: package.to_path_buf() }
    }

    #[must_use]
    pub fn package(&self) -> &Path {
        &self.package
    }

    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.tree
    }
}
