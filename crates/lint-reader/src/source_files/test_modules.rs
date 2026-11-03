use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::modules::{Child, Modules};
use super::scope::Scope;
use super::shelf::Shelf;
use lint_core::SourceFile;

#[derive(Debug)]
pub struct TestModules<'shelf, 'tree> {
    shelf: &'shelf Shelf<'tree>,
    seen: BTreeSet<PathBuf>,
    files: Vec<SourceFile>,
}

impl<'shelf, 'tree> TestModules<'shelf, 'tree> {
    #[must_use]
    pub fn read(shelf: &'shelf Shelf<'tree>, scope: &Scope<'tree>, children: &[Child]) -> Vec<SourceFile> {
        let started = TestModules { shelf, seen: BTreeSet::new(), files: Vec::new() };
        children.iter().cloned().fold(started, |reading, child| reading.child(scope, child)).files
    }

    fn child(self, scope: &Scope<'tree>, child: Child) -> TestModules<'shelf, 'tree> {
        child.canonical(self.shelf.survey()).visit(|path, directory| self.file(scope, path, directory))
    }

    fn file(mut self, scope: &Scope<'tree>, file: &Path, children_directory: &Path) -> TestModules<'shelf, 'tree> {
        if self.seen.insert(file.to_path_buf()) { self.opened(scope, file, children_directory) } else { self }
    }

    fn opened(self, scope: &Scope<'tree>, file: &Path, children_directory: &Path) -> TestModules<'shelf, 'tree> {
        match self.shelf.test_source(scope, file) {
            Some(source) => {
                let nested =
                    Modules::declared_in(self.shelf.survey(), file, children_directory, &source.syntax().items)
                        .nested();
                nested.into_iter().fold(
                    TestModules { files: self.files.into_iter().chain([source]).collect(), ..self },
                    |reading, child| reading.child(scope, child),
                )
            }
            None => self,
        }
    }
}
