use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::modules::Modules;
use crate::readout::Readout;
use crate::roots::Roots;
use crate::source_file::SourceFile;
use crate::tree::Tree;

#[derive(Debug, Default)]
pub struct SourceFiles {
    seen: BTreeSet<PathBuf>,
    readout: Readout,
}

impl SourceFiles {
    pub fn read(root: &Path) -> Result<Readout, Failure> {
        Self::read_all(&[root.to_path_buf()])
    }

    pub fn read_all(roots: &[PathBuf]) -> Result<Readout, Failure> {
        let mut reading = SourceFiles::default();
        for root in roots {
            reading.tree(&Tree::open(root)?)?;
        }
        reading.readout.sort();
        Ok(reading.readout)
    }

    fn tree(&mut self, tree: &Tree) -> Result<(), Failure> {
        let roots = Roots::under(tree)?;
        if roots.is_empty() {
            return Err(Failure::NoSources { path: tree.given().to_path_buf() });
        }
        for root in roots {
            let children = root.parent().map(Path::to_path_buf).unwrap_or_default();
            self.file(tree, &root, &children);
        }
        Ok(())
    }

    fn file(&mut self, tree: &Tree, file: &Path, children_directory: &Path) {
        let shown = tree.display(file);
        if !self.seen.insert(file.to_path_buf()) {
            return;
        }
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(source) => return self.readout.add_failure(Failure::Io { path: shown, source }),
        };
        match SourceFile::parse_at(shown, file, text) {
            Ok(parsed) => {
                let mut modules = Modules::declared_in(file, children_directory, &parsed.syntax().items);
                self.readout.add_file(parsed);
                for failure in modules.take_failures() {
                    self.readout.add_failure(failure);
                }
                for child in modules.take_children() {
                    let canonical = fs::canonicalize(child.path()).unwrap_or_else(|_| child.path().to_path_buf());
                    self.file(tree, &canonical, child.children_directory());
                }
            }
            Err(failure) => self.readout.add_failure(failure),
        }
    }
}
