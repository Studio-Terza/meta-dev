mod modules;
mod roots;
mod tree;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::readout::Readout;
use crate::source_file::SourceFile;
use modules::Modules;
use roots::Roots;
use tree::Tree;

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
        roots
            .iter()
            .try_fold(SourceFiles::default(), |reading, root| reading.tree(&Tree::open(root)?))
            .map(|reading| reading.readout.sorted())
    }

    fn tree(self, tree: &Tree) -> Result<SourceFiles, Failure> {
        let roots = Roots::under(tree)?;
        if roots.is_empty() {
            Err(tree.without_sources())
        } else {
            Ok(roots.into_iter().fold(self, |reading, root| {
                let children = root.parent().map(Path::to_path_buf).unwrap_or_default();
                reading.file(tree, &root, &children)
            }))
        }
    }

    fn file(mut self, tree: &Tree, file: &Path, children_directory: &Path) -> SourceFiles {
        if self.seen.insert(file.to_path_buf()) { self.opened(tree, file, children_directory) } else { self }
    }

    fn opened(self, tree: &Tree, file: &Path, children_directory: &Path) -> SourceFiles {
        let shown = tree.display(file);
        let read = fs::read_to_string(file)
            .map_err(|source| Failure::Io { path: shown.clone(), source })
            .and_then(|text| SourceFile::parse_at(shown, file, text));
        match read {
            Ok(parsed) => self.parsed(tree, file, children_directory, parsed),
            Err(failure) => SourceFiles { readout: self.readout.with_failure(failure), ..self },
        }
    }

    fn parsed(self, tree: &Tree, file: &Path, children_directory: &Path, parsed: SourceFile) -> SourceFiles {
        let mut modules = Modules::declared_in(file, children_directory, &parsed.syntax().items);
        let with_file = SourceFiles { readout: self.readout.with_file(parsed), ..self };
        let with_failures = modules.take_failures().into_iter().fold(with_file, |reading, failure| SourceFiles {
            readout: reading.readout.with_failure(failure),
            ..reading
        });
        modules.take_children().into_iter().fold(with_failures, |reading, child| {
            child.canonically(|path, directory| reading.file(tree, path, directory))
        })
    }
}
