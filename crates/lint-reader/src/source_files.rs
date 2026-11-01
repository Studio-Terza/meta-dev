mod modules;
mod scope;
mod test_modules;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use lint_core::SourceFile;
use lint_core::{Readout, SourceText};
use lint_layout::Failure;
use lint_placement::CrateFile;
use lint_roots::Roots;
use lint_roots::Tree;
use modules::Modules;
use scope::Scope;
use test_modules::TestModules;

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
                let package = root.package().to_path_buf();
                let file = root.into_file();
                let scope = Scope::new(tree, &package, CrateFile::from(tree.display(&file)));
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                reading.file(&scope, &file, &children)
            }))
        }
    }

    fn file(mut self, scope: &Scope<'_>, file: &Path, children_directory: &Path) -> SourceFiles {
        if self.seen.insert(file.to_path_buf()) { self.opened(scope, file, children_directory) } else { self }
    }

    fn opened(self, scope: &Scope<'_>, file: &Path, children_directory: &Path) -> SourceFiles {
        let shown = scope.tree().display(file);
        let read = fs::read_to_string(file)
            .map(SourceText::new)
            .map_err(|source| Failure::Io { path: shown.clone(), source })
            .and_then(|text| SourceFile::parse_at(shown, scope.package(), file, text))
            .map(|parsed| parsed.in_crate_file(scope.crate_file().clone()));
        match read {
            Ok(parsed) => self.parsed(scope, file, children_directory, parsed),
            Err(failure) => SourceFiles { readout: self.readout.with_failure(failure), ..self },
        }
    }

    fn parsed(self, scope: &Scope<'_>, file: &Path, children_directory: &Path, parsed: SourceFile) -> SourceFiles {
        let mut modules = Modules::declared_in(file, children_directory, &parsed.syntax().items);
        let tests = TestModules::read(scope, modules.tests());
        let with_file = SourceFiles { readout: self.readout.with_file(parsed.with_test_modules(tests)), ..self };
        let with_failures = modules.take_failures().into_iter().fold(with_file, |reading, failure| SourceFiles {
            readout: reading.readout.with_failure(failure),
            ..reading
        });
        modules.take_children().into_iter().fold(with_failures, |reading, child| {
            child.canonically(|path, directory| reading.file(scope, path, directory))
        })
    }
}
