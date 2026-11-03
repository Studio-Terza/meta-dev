use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_placement::CrateFile;
use lint_roots::Tree;

#[derive(Debug)]
pub struct Scope<'tree, Port> {
    files: &'tree Port,
    tree: &'tree Tree,
    package: PathBuf,
    crate_file: CrateFile,
}

impl<'tree, Port: Files> Scope<'tree, Port> {
    #[must_use]
    pub fn new(files: &'tree Port, tree: &'tree Tree, package: &Path, crate_file: CrateFile) -> Scope<'tree, Port> {
        Scope { files, tree, package: package.to_path_buf(), crate_file }
    }

    #[must_use]
    pub fn files(&self) -> &'tree Port {
        self.files
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
