use std::path::{Path, PathBuf};

use super::tree::Tree;
use crate::crate_file::CrateFile;

#[derive(Debug)]
pub struct Scope<'tree> {
    tree: &'tree Tree,
    package: PathBuf,
    crate_file: CrateFile,
}

impl<'tree> Scope<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree, package: &Path, crate_file: CrateFile) -> Scope<'tree> {
        Scope { tree, package: package.to_path_buf(), crate_file }
    }

    #[must_use]
    pub fn crate_file(&self) -> &CrateFile {
        &self.crate_file
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
