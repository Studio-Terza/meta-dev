mod child;
mod declaration;
mod redirects;

use std::io;
use std::path::{Path, PathBuf};

use syn::Item;

use lint_core::Walk;
use lint_layout::Failure;
use lint_layout::Name;
use lint_roots::Survey;

pub use child::Child;
use declaration::Declaration;
use redirects::Redirects;

#[derive(Debug)]
pub struct Modules {
    children: Vec<Child>,
    tests: Vec<Child>,
    failures: Vec<Failure>,
}

impl Modules {
    #[must_use]
    pub fn candidates(file: &Path, children_directory: &Path, items: &[Item]) -> Vec<PathBuf> {
        let file_directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
        [Walk::Entered, Walk::Skipped]
            .into_iter()
            .flat_map(|wanted| Self::declarations(items, children_directory, wanted))
            .flat_map(|declaration| {
                declaration
                    .choices(&file_directory)
                    .into_iter()
                    .chain(Redirects::conditional(&file_directory, declaration.module()))
            })
            .map(|child| child.as_ref().to_path_buf())
            .collect()
    }

    #[must_use]
    pub fn declared_in(survey: &Survey, file: &Path, children_directory: &Path, items: &[Item]) -> Modules {
        let file_directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let entered = Self::declarations(items, children_directory, Walk::Entered);
        let skipped = Self::declarations(items, children_directory, Walk::Skipped);
        let tests = skipped
            .iter()
            .flat_map(|declaration| Self::resolve(survey, &file_directory, declaration).unwrap_or_default())
            .collect();
        entered.into_iter().fold(
            Modules { children: Vec::new(), tests, failures: Vec::new() },
            |modules, declaration| match Self::resolve(survey, &file_directory, &declaration) {
                Ok(found) => Modules { children: modules.children.into_iter().chain(found).collect(), ..modules },
                Err(failure) => {
                    Modules { failures: modules.failures.into_iter().chain([failure]).collect(), ..modules }
                }
            },
        )
    }

    pub fn take_children(&mut self, survey: &Survey) -> Vec<Child> {
        std::mem::take(&mut self.children).into_iter().map(|child| child.canonical(survey)).collect()
    }

    #[must_use]
    pub fn nested(self) -> Vec<Child> {
        self.children.into_iter().chain(self.tests).collect()
    }

    #[must_use]
    pub fn tests(&self) -> &[Child] {
        &self.tests
    }

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    fn declarations<'items>(items: &'items [Item], directory: &Path, wanted: Walk) -> Vec<Declaration<'items>> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) => Some(module),
                _ => None,
            })
            .flat_map(|module| match (&module.content, Walk::outside_tests(&module.attrs)) {
                (Some((_, inner)), Walk::Entered) => {
                    Self::declarations(inner, &directory.join(Name::from(&module.ident)), wanted)
                }
                (None, walk) if walk == wanted => vec![Declaration::new(module, directory.to_path_buf())],
                _ => Vec::new(),
            })
            .collect()
    }

    fn resolve(survey: &Survey, file_directory: &Path, declaration: &Declaration<'_>) -> Result<Vec<Child>, Failure> {
        let primary =
            declaration.choices(file_directory).into_iter().find(|choice| survey.file(choice.as_ref()).is_some());
        let found: Vec<Child> = primary.iter().cloned().collect();
        let found = Redirects::conditional(file_directory, declaration.module())
            .into_iter()
            .filter(|child| survey.file(child.as_ref()).is_some())
            .fold(
                found,
                |found, child| {
                    if found.contains(&child) { found } else { found.into_iter().chain([child]).collect() }
                },
            );
        if found.is_empty() {
            Err(Failure::Io {
                path: declaration.expected(file_directory),
                source: io::Error::from(io::ErrorKind::NotFound),
            })
        } else {
            Ok(found)
        }
    }
}
