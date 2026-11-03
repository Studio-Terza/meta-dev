use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::modules::{Child, Modules};
use super::scope::Scope;
use file_ports::Files;
use lint_core::{SourceFile, SourceText};
use lint_placement::Placement;
use newtype::Newtype;

#[derive(Debug, Default)]
pub struct TestModules {
    seen: BTreeSet<PathBuf>,
    files: Vec<SourceFile>,
}

impl TestModules {
    #[must_use]
    pub fn read(scope: &Scope<'_, impl Files>, children: &[Child]) -> Vec<SourceFile> {
        children.iter().cloned().fold(TestModules::default(), |reading, child| reading.child(scope, child)).files
    }

    fn child(self, scope: &Scope<'_, impl Files>, child: Child) -> TestModules {
        child.canonically(scope.files(), |path, directory| self.file(scope, path, directory))
    }

    fn file(mut self, scope: &Scope<'_, impl Files>, file: &Path, children_directory: &Path) -> TestModules {
        if self.seen.insert(file.to_path_buf()) { self.opened(scope, file, children_directory) } else { self }
    }

    fn opened(self, scope: &Scope<'_, impl Files>, file: &Path, children_directory: &Path) -> TestModules {
        let parsed =
            scope.files().read_text(file).ok().map(|text| SourceText::new(text.into_inner())).and_then(|text| {
                let placement = Placement::within(scope.files(), scope.package(), file);
                SourceFile::parse_at(scope.tree().display(file), file.to_path_buf(), placement, text).ok()
            });
        match parsed {
            Some(source) => {
                let mut modules = Modules::declared_in(scope.files(), file, children_directory, &source.syntax().items);
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
