use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use lint_core::{Readout, RepoFile, SourceFile};
use lint_layout::Failure;
use lint_placement::CrateFile;
use lint_roots::{Survey, Tree};

use super::boundary::Expected;
use super::boundary::Partition;
use super::declared::{Declared, Explored};
use crate::source_files::Stage;
use crate::source_files::located::Located;
use crate::source_files::modules::Child;
use crate::source_files::scope::Scope;
use crate::source_files::staged::Staged;

#[derive(Debug)]
struct MainFile<'tree> {
    crate_file: CrateFile,
    tests: Vec<Located<'tree>>,
}

#[derive(Debug, Default)]
pub(super) struct Assembly<'tree> {
    mains: BTreeMap<Located<'tree>, MainFile<'tree>>,
    failures: Mutex<Vec<Failure>>,
    repo_files: Vec<RepoFile>,
}

impl<'tree> Assembly<'tree> {
    pub(super) fn walked(
        stage: &'tree Stage<'tree>,
        survey: Survey,
        explored: BTreeMap<Located<'tree>, Explored>,
    ) -> Assembly<'tree> {
        let walk = Walk { survey, explored, seen: BTreeSet::new(), assembly: Assembly::default() };
        let roots = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().map(move |entry| Root { tree, entry }));
        roots.fold(walk, Walk::rooted).assembly
    }

    pub(super) fn expected(&self) -> Expected {
        Expected::counted(self.mains.len())
    }

    pub(super) fn readout(&self, parsed: BTreeMap<Located<'tree>, SourceFile>, share: Partition) -> Readout {
        let mut local = parsed;
        let mains: Vec<SourceFile> =
            self.mains.iter().filter_map(|(located, main)| main.taken(located, &mut local)).collect();
        let failures = Some(share)
            .filter(|found| *found == Partition::MAIN)
            .map_or_default(|_| mem::take(&mut *self.failures.lock().unwrap_or_else(PoisonError::into_inner)));
        let with_files =
            mains.into_iter().fold(Readout::default().with_repo_files(self.repo_files.clone()), Readout::with_file);
        failures.into_iter().fold(with_files, Readout::with_failure).sorted()
    }

    fn main(mut self, located: Located<'tree>, main: MainFile<'tree>) -> Assembly<'tree> {
        self.mains.insert(located, main);
        self
    }

    fn failed(mut self, failures: Vec<Failure>) -> Assembly<'tree> {
        self.failures.get_mut().unwrap_or_else(PoisonError::into_inner).extend(failures);
        self
    }
}

impl<'tree> MainFile<'tree> {
    fn taken(&self, located: &Located<'tree>, local: &mut BTreeMap<Located<'tree>, SourceFile>) -> Option<SourceFile> {
        let file = local.remove(located)?;
        let tests = self.tests.iter().filter_map(|test| local.remove(test)).collect();
        Some(file.in_crate_file(self.crate_file.clone()).with_test_modules(tests))
    }
}

#[derive(Clone, Copy)]
struct Root<'tree> {
    tree: &'tree Tree<'tree>,
    entry: &'tree Staged,
}

struct Walk<'tree> {
    survey: Survey,
    explored: BTreeMap<Located<'tree>, Explored>,
    seen: BTreeSet<PathBuf>,
    assembly: Assembly<'tree>,
}

impl<'tree> Walk<'tree> {
    fn rooted(self, root: Root<'tree>) -> Walk<'tree> {
        let tree = root.tree;
        let starts = root.entry.targets().iter().flat_map(|target| {
            target
                .files()
                .iter()
                .map(move |file| (Scope::new(tree, target.package(), CrateFile::from(tree.display(file))), file))
        });
        let walked = starts.fold(self, |walk, (scope, file)| walk.file(&scope, file));
        let repo_files = walked.assembly.repo_files.iter().chain(root.entry.repo_files()).cloned().collect();
        Walk { assembly: Assembly { repo_files, ..walked.assembly }, ..walked }
    }

    fn file(mut self, scope: &Scope<'tree>, file: &Path) -> Walk<'tree> {
        let located = Located::new(scope.tree(), scope.package(), file);
        let fresh = self.seen.insert(file.to_path_buf()).then_some(&located);
        let explored = fresh.and_then(|unseen| self.explored.remove(unseen));
        explored.into_iter().fold(self, |walk, found| walk.found(scope, &located, found))
    }

    fn found(self, scope: &Scope<'tree>, located: &Located<'tree>, found: Explored) -> Walk<'tree> {
        match found {
            Ok(declared) => self.declared(scope, located, declared),
            Err(failure) => Walk { assembly: self.assembly.failed(vec![failure]), ..self },
        }
    }

    fn declared(self, scope: &Scope<'tree>, located: &Located<'tree>, mut declared: Declared) -> Walk<'tree> {
        let testing = declared
            .nested(|child, kind| kind.tested(child))
            .into_iter()
            .flatten()
            .fold(Testing { walk: self, seen: BTreeSet::new(), found: Vec::new() }, |reading, child| {
                reading.child(scope, child)
            });
        let main = MainFile { crate_file: scope.crate_file().clone(), tests: testing.found };
        let assembly = testing.walk.assembly.main(located.clone(), main).failed(declared.failures());
        let walk = Walk { assembly, ..testing.walk };
        declared
            .into_children()
            .into_iter()
            .fold(walk, |walking, child| child.visit(|path, _| walking.file(scope, path)))
    }
}

struct Testing<'tree> {
    walk: Walk<'tree>,
    seen: BTreeSet<PathBuf>,
    found: Vec<Located<'tree>>,
}

impl<'tree> Testing<'tree> {
    fn child(self, scope: &Scope<'tree>, child: Child) -> Testing<'tree> {
        child.canonical(&self.walk.survey).visit(|path, _| self.file(scope, path))
    }

    fn file(mut self, scope: &Scope<'tree>, file: &Path) -> Testing<'tree> {
        let located = Located::new(scope.tree(), scope.package(), file);
        let fresh = self.seen.insert(file.to_path_buf()).then_some(&located);
        let declared = fresh.and_then(|unseen| self.walk.explored.remove(unseen)).and_then(Result::ok);
        declared.into_iter().fold(self, |testing, found| testing.nested(scope, &located, &found))
    }

    fn nested(mut self, scope: &Scope<'tree>, located: &Located<'tree>, declared: &Declared) -> Testing<'tree> {
        self.found.push(located.clone());
        declared.nested(|child, _| child.clone()).into_iter().fold(self, |testing, nested| testing.child(scope, nested))
    }
}
