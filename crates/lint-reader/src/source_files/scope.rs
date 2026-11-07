use std::path::{Path, PathBuf};

use lint_placement::CrateFile;
use lint_roots::Tree;

#[derive(Debug)]
pub struct Scope<'tree> {
    tree: &'tree Tree<'tree>,
    package: PathBuf,
    crate_file: CrateFile,
}

impl<'tree> Scope<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree<'tree>, package: &Path, crate_file: CrateFile) -> Self {
        Scope { tree, package: package.to_path_buf(), crate_file }
    }

    #[must_use]
    pub const fn tree(&self) -> &'tree Tree<'tree> {
        self.tree
    }

    #[must_use]
    pub fn package(&self) -> &Path {
        &self.package
    }

    #[must_use]
    pub const fn crate_file(&self) -> &CrateFile {
        &self.crate_file
    }
}
