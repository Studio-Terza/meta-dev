mod located;
mod modules;
mod opened;
mod scope;
mod shelf;
mod staged;
mod test_modules;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_core::{Readout, SourceFile};
use lint_layout::Failure;
use lint_placement::{CrateFile, Packages};
use lint_roots::{Regrouped, Target, Tree};
use modules::Modules;
use scope::Scope;
use shelf::Shelf;
use staged::Staged;
use test_modules::TestModules;

#[derive(Debug)]
pub struct Stage<'given> {
    trees: Vec<Tree<'given>>,
    staged: Vec<Vec<Staged>>,
    packages: Packages,
}

#[derive(Debug)]
pub struct SourceFiles<'tree> {
    seen: BTreeSet<PathBuf>,
    readout: Readout,
    shelf: Shelf<'tree>,
}

impl<'tree> SourceFiles<'tree> {
    pub fn read(files: &(impl Files + Sync), root: &Path) -> Result<Readout, Failure> {
        SourceFiles::read_all(files, &[root.to_path_buf()])
    }

    pub fn read_all(files: &(impl Files + Sync), roots: &[PathBuf]) -> Result<Readout, Failure> {
        SourceFiles::staged(files, roots).map(|stage| SourceFiles::read_staged(files, &stage))
    }

    pub fn staged<'given>(files: &(impl Files + Sync), roots: &'given [PathBuf]) -> Result<Stage<'given>, Failure> {
        let opened = Tree::open_many(files, roots);
        let shape: Vec<Vec<&Tree<'_>>> = opened.iter().map(|opened| opened.iter().collect()).collect();
        let staged = Regrouped::like(&shape, Staged::under(files, &shape.concat()));
        let (trees, staged): (Vec<Tree<'_>>, Vec<Vec<Staged>>) = opened
            .into_iter()
            .zip(staged)
            .map(|(opened, staged)| Ok((opened?, staged.into_iter().collect::<Result<Vec<Staged>, Failure>>()?)))
            .collect::<Result<Vec<(Tree<'_>, Vec<Staged>)>, Failure>>()?
            .into_iter()
            .unzip();
        let packages: Vec<&Path> = staged.iter().flatten().flat_map(Staged::targets).map(Target::package).collect();
        Ok(Stage { packages: Packages::read(files, &packages), trees, staged })
    }

    #[must_use]
    pub fn read_staged(files: &impl Files, stage: &'tree Stage<'tree>) -> Readout {
        let starts = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().flat_map(|staged| staged.starts(tree)));
        let started = SourceFiles {
            seen: BTreeSet::new(),
            readout: Readout::default(),
            shelf: Shelf::stocked(files, &stage.packages, &starts.collect::<Vec<_>>()),
        };
        let read = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().map(move |staged| (tree, staged)));
        read.fold(started, |reading, (tree, staged)| reading.tree(tree, staged)).readout.sorted()
    }

    fn tree(self, tree: &'tree Tree<'tree>, staged: &Staged) -> Self {
        let read = staged.targets().iter().fold(self, |reading, target| {
            target.files().iter().fold(reading, |reading, file| {
                let scope = Scope::new(tree, target.package(), CrateFile::from(tree.display(file)));
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                reading.file(&scope, file, &children)
            })
        });
        SourceFiles { readout: read.readout.with_repo_files(staged.repo_files().to_vec()), ..read }
    }

    fn file(mut self, scope: &Scope<'tree>, file: &Path, children_directory: &Path) -> Self {
        let source =
            self.seen.insert(file.to_path_buf()).then_some(file).and_then(|file| self.shelf.source(scope, file));
        self.opened(scope, file, children_directory, source)
    }

    fn opened(
        self,
        scope: &Scope<'tree>,
        file: &Path,
        children_directory: &Path,
        source: Option<Result<SourceFile, Failure>>,
    ) -> Self {
        match source {
            Some(Ok(parsed)) => {
                self.parsed(scope, file, children_directory, parsed.in_crate_file(scope.crate_file().clone()))
            }
            Some(Err(failure)) => SourceFiles { readout: self.readout.with_failure(failure), ..self },
            None => self,
        }
    }

    fn parsed(self, scope: &Scope<'tree>, file: &Path, children_directory: &Path, parsed: SourceFile) -> Self {
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
