mod located;
mod modules;
mod scope;
mod shelf;
mod staged;
mod test_modules;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_core::{Readout, SourceFile};
use lint_layout::Failure;
use lint_placement::CrateFile;
use lint_roots::{Regrouped, Tree};
use modules::Modules;
use scope::Scope;
use shelf::Shelf;
use staged::Staged;
use test_modules::TestModules;

#[derive(Debug)]
pub struct SourceFiles<'tree> {
    seen: BTreeSet<PathBuf>,
    readout: Readout,
    shelf: Shelf<'tree>,
}

impl<'tree> SourceFiles<'tree> {
    pub fn read(files: &impl Files, root: &Path) -> Result<Readout, Failure> {
        SourceFiles::read_all(files, &[root.to_path_buf()])
    }

    pub fn read_all(files: &impl Files, roots: &[PathBuf]) -> Result<Readout, Failure> {
        let opened = Tree::open_many(files, roots);
        let shape: Vec<Vec<&Tree>> = opened.iter().map(|opened| opened.iter().collect()).collect();
        let staged = Regrouped::like(&shape, Staged::under(files, &shape.concat()));
        let (trees, staged): (Vec<Tree>, Vec<Vec<Staged>>) = opened
            .into_iter()
            .zip(staged)
            .map(|(opened, staged)| Ok((opened?, staged.into_iter().collect::<Result<Vec<Staged>, Failure>>()?)))
            .collect::<Result<Vec<(Tree, Vec<Staged>)>, Failure>>()?
            .into_iter()
            .unzip();
        let starts =
            trees.iter().zip(&staged).flat_map(|(tree, staged)| staged.iter().flat_map(|staged| staged.starts(tree)));
        let started = SourceFiles {
            seen: BTreeSet::new(),
            readout: Readout::default(),
            shelf: Shelf::stocked(files, &starts.collect::<Vec<_>>()),
        };
        let read =
            trees.iter().zip(staged).flat_map(|(tree, staged)| staged.into_iter().map(move |staged| (tree, staged)));
        Ok(read.fold(started, |reading, (tree, staged)| reading.tree(tree, staged)).readout.sorted())
    }

    fn tree(self, tree: &'tree Tree, staged: Staged) -> SourceFiles<'tree> {
        let targets = staged.targets().to_vec();
        let started = SourceFiles { readout: self.readout.with_repo_files(staged.into_repo_files()), ..self };
        targets.into_iter().fold(started, |reading, root| {
            let package = root.package().to_path_buf();
            let file = root.into_file();
            let scope = Scope::new(tree, &package, CrateFile::from(tree.display(&file)));
            let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
            reading.file(&scope, &file, &children)
        })
    }

    fn file(mut self, scope: &Scope<'tree>, file: &Path, children_directory: &Path) -> SourceFiles<'tree> {
        if self.seen.insert(file.to_path_buf()) {
            let source = self.shelf.source(scope, file);
            self.opened(scope, file, children_directory, source)
        } else {
            self
        }
    }

    fn opened(
        self,
        scope: &Scope<'tree>,
        file: &Path,
        children_directory: &Path,
        source: Option<Result<SourceFile, Failure>>,
    ) -> SourceFiles<'tree> {
        match source {
            Some(Ok(parsed)) => {
                self.parsed(scope, file, children_directory, parsed.in_crate_file(scope.crate_file().clone()))
            }
            Some(Err(failure)) => SourceFiles { readout: self.readout.with_failure(failure), ..self },
            None => self,
        }
    }

    fn parsed(
        self,
        scope: &Scope<'tree>,
        file: &Path,
        children_directory: &Path,
        parsed: SourceFile,
    ) -> SourceFiles<'tree> {
        let mut modules = Modules::declared_in(self.shelf.survey(), file, children_directory, &parsed.syntax().items);
        let tests = TestModules::read(&self.shelf, scope, modules.tests());
        let children = modules.take_children(self.shelf.survey());
        let with_file = SourceFiles { readout: self.readout.with_file(parsed.with_test_modules(tests)), ..self };
        let with_failures = modules.take_failures().into_iter().fold(with_file, |reading, failure| SourceFiles {
            readout: reading.readout.with_failure(failure),
            ..reading
        });
        children
            .into_iter()
            .fold(with_failures, |reading, child| child.visit(|path, directory| reading.file(scope, path, directory)))
    }
}
