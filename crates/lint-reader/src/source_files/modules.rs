mod child;
mod declaration;
mod redirects;

use std::io;
use std::path::{Path, PathBuf};

use syn::Item;

use lint_core::Walk;
use lint_layout::Failure;
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
        let file_directory = file.parent().map_or_default(Path::to_path_buf);
        [Walk::Entered, Walk::Skipped]
            .into_iter()
            .flat_map(|wanted| Declaration::within(items, children_directory, wanted))
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
    pub fn nested_in(survey: &Survey, file: &Path, children_directory: &Path, items: &[Item]) -> Vec<Child> {
        let declared = Self::declared_in(survey, file, children_directory, items);
        declared.children.into_iter().chain(declared.tests).collect()
    }

    #[must_use]
    pub fn declared_in(survey: &Survey, file: &Path, children_directory: &Path, items: &[Item]) -> Modules {
        let file_directory = file.parent().map_or_default(Path::to_path_buf);
        let entered = Declaration::within(items, children_directory, Walk::Entered);
        let skipped = Declaration::within(items, children_directory, Walk::Skipped);
        let tests = skipped
            .iter()
            .flat_map(|declaration| Self::resolve(survey, &file_directory, declaration).unwrap_or_default())
            .collect();
        entered
            .into_iter()
            .fold(Modules { children: Vec::new(), tests, failures: Vec::new() }, |modules, declaration| {
                modules.resolved_into(Self::resolve(survey, &file_directory, &declaration))
            })
    }

    pub fn take_children(&mut self, survey: &Survey) -> Vec<Child> {
        std::mem::take(&mut self.children).into_iter().map(|child| child.canonical(survey)).collect()
    }

    #[must_use]
    pub fn tests(&self) -> &[Child] {
        &self.tests
    }

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    fn resolved_into(self, resolved: Result<Vec<Child>, Failure>) -> Modules {
        match resolved {
            Ok(found) => Modules { children: self.children.into_iter().chain(found).collect(), ..self },
            Err(failure) => Modules { failures: self.failures.into_iter().chain([failure]).collect(), ..self },
        }
    }

    fn resolve(survey: &Survey, file_directory: &Path, declaration: &Declaration<'_>) -> Result<Vec<Child>, Failure> {
        let primary =
            declaration.choices(file_directory).into_iter().find(|choice| survey.file(choice.as_ref()).is_some());
        let candidates: Vec<Child> = primary
            .into_iter()
            .chain(
                Redirects::conditional(file_directory, declaration.module())
                    .into_iter()
                    .filter(|child| survey.file(child.as_ref()).is_some()),
            )
            .collect();
        let firsts: Vec<Option<usize>> = candidates
            .iter()
            .enumerate()
            .map(|(index, child)| candidates.iter().position(|other| other == child).filter(|first| *first == index))
            .collect();
        let found: Vec<Child> =
            candidates.into_iter().zip(firsts).filter_map(|(child, first)| first.map(|_| child)).collect();
        found.first().map(|_| ()).map(|()| found).ok_or_else(|| Failure::Io {
            path: declaration.expected(file_directory),
            source: io::Error::from(io::ErrorKind::NotFound),
        })
    }
}
