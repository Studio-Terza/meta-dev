use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::modules::{Child, Modules};
use super::scope::Scope;
use crate::source_file::SourceFile;

#[derive(Debug, Default)]
pub struct TestModules {
    seen: BTreeSet<PathBuf>,
    files: Vec<SourceFile>,
}

impl TestModules {
    #[must_use]
    pub fn read(scope: &Scope<'_>, children: &[Child]) -> Vec<SourceFile> {
        children.iter().cloned().fold(TestModules::default(), |reading, child| reading.child(scope, child)).files
    }

    fn child(self, scope: &Scope<'_>, child: Child) -> TestModules {
        child.canonically(|path, directory| self.file(scope, path, directory))
    }

    fn file(mut self, scope: &Scope<'_>, file: &Path, children_directory: &Path) -> TestModules {
        if self.seen.insert(file.to_path_buf()) { self.opened(scope, file, children_directory) } else { self }
    }

    fn opened(self, scope: &Scope<'_>, file: &Path, children_directory: &Path) -> TestModules {
        let parsed = fs::read_to_string(file)
            .ok()
            .and_then(|text| SourceFile::parse_at(scope.tree().display(file), scope.package(), file, text).ok());
        match parsed {
            Some(source) => {
                let mut modules = Modules::declared_in(file, children_directory, &source.syntax().items);
                let nested: Vec<Child> =
                    modules.take_children().into_iter().chain(modules.tests().iter().cloned()).collect();
                nested.into_iter().fold(
                    TestModules { files: self.files.into_iter().chain([source]).collect(), ..self },
                    |reading, child| reading.child(scope, child),
                )
            }
            None => self,
        }
    }
}
