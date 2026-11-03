mod modules;
mod repo_files;
mod scope;
mod test_modules;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_core::SourceFile;
use lint_core::{Readout, SourceText};
use lint_layout::Failure;
use lint_placement::{CrateFile, Placement};
use lint_roots::Roots;
use lint_roots::Tree;
use modules::Modules;
use newtype::Newtype;
use repo_files::RepoFiles;
use scope::Scope;
use test_modules::TestModules;

#[derive(Debug, Default)]
pub struct SourceFiles {
    seen: BTreeSet<PathBuf>,
    readout: Readout,
}

impl SourceFiles {
    pub fn read(files: &impl Files, root: &Path) -> Result<Readout, Failure> {
        Self::read_all(files, &[root.to_path_buf()])
    }

    pub fn read_all(files: &impl Files, roots: &[PathBuf]) -> Result<Readout, Failure> {
        roots
            .iter()
            .try_fold(SourceFiles::default(), |reading, root| reading.tree(files, &Tree::open(files, root)?))
            .map(|reading| reading.readout.sorted())
    }

    fn tree(self, files: &impl Files, tree: &Tree) -> Result<SourceFiles, Failure> {
        let roots = Roots::under(files, tree)?;
        if roots.is_empty() {
            Err(tree.without_sources())
        } else {
            let repo_files = RepoFiles::under(files, tree)?;
            let started = SourceFiles { readout: self.readout.with_repo_files(repo_files), ..self };
            Ok(roots.into_iter().fold(started, |reading, root| {
                let package = root.package().to_path_buf();
                let file = root.into_file();
                let scope = Scope::new(files, tree, &package, CrateFile::from(tree.display(&file)));
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                reading.file(&scope, &file, &children)
            }))
        }
    }

    fn file(mut self, scope: &Scope<'_, impl Files>, file: &Path, children_directory: &Path) -> SourceFiles {
        if self.seen.insert(file.to_path_buf()) { self.opened(scope, file, children_directory) } else { self }
    }

    fn opened(self, scope: &Scope<'_, impl Files>, file: &Path, children_directory: &Path) -> SourceFiles {
        let shown = scope.tree().display(file);
        let read = scope
            .files()
            .read_text(file)
            .map(|text| SourceText::new(text.into_inner()))
            .map_err(|failure| Failure::Io { path: shown.clone(), source: failure.into() })
            .and_then(|text| {
                let placement = Placement::within(scope.files(), scope.package(), file);
                SourceFile::parse_at(shown, file.to_path_buf(), placement, text)
            })
            .map(|parsed| parsed.in_crate_file(scope.crate_file().clone()));
        match read {
            Ok(parsed) => self.parsed(scope, file, children_directory, parsed),
            Err(failure) => SourceFiles { readout: self.readout.with_failure(failure), ..self },
        }
    }

    fn parsed(
        self,
        scope: &Scope<'_, impl Files>,
        file: &Path,
        children_directory: &Path,
        parsed: SourceFile,
    ) -> SourceFiles {
        let mut modules = Modules::declared_in(scope.files(), file, children_directory, &parsed.syntax().items);
        let tests = TestModules::read(scope, modules.tests());
        let with_file = SourceFiles { readout: self.readout.with_file(parsed.with_test_modules(tests)), ..self };
        let with_failures = modules.take_failures().into_iter().fold(with_file, |reading, failure| SourceFiles {
            readout: reading.readout.with_failure(failure),
            ..reading
        });
        modules.take_children().into_iter().fold(with_failures, |reading, child| {
            child.canonically(scope.files(), |path, directory| reading.file(scope, path, directory))
        })
    }
}
