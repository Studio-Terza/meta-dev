mod child;
mod declaration;
mod redirects;

use std::io;
use std::path::{Path, PathBuf};

use syntax::Item;

use lint_core::Walk;
use lint_layout::Failure;
use lint_roots::Survey;

use super::declared::Resolution;
use crate::source_files::ignored::Ignored;
pub use child::Child;
use declaration::Declaration;
use invariants::Ignoring;
use redirects::Redirects;

#[derive(Clone, Debug)]
pub struct Sought {
    walk: Walk,
    choices: Vec<Child>,
    conditional: Vec<Child>,
    expected: PathBuf,
}

impl Sought {
    #[must_use]
    pub fn within(file: &Path, children_directory: &Path, items: &[Item]) -> Vec<Sought> {
        let file_directory = file.parent().map_or_default(Path::to_path_buf);
        [Walk::Entered, Walk::Skipped]
            .into_iter()
            .flat_map(|wanted| {
                Declaration::within(items, children_directory, wanted)
                    .into_iter()
                    .map(move |declaration| (wanted, declaration))
            })
            .map(|(walk, declaration)| Sought {
                walk,
                choices: declaration.choices(&file_directory),
                conditional: Redirects::conditional(&file_directory, declaration.module()),
                expected: declaration.expected(&file_directory),
            })
            .collect()
    }

    pub fn candidates(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.choices.iter().chain(&self.conditional).map(|child| child.as_ref().to_path_buf())
    }

    pub(super) fn resolution(&self, survey: &Survey) -> Option<Resolution> {
        match self.walk {
            Walk::Entered => Some(Resolution::from(self.resolved(survey))),
            Walk::Skipped => self.resolved(survey).ignoring(Ignored::Unresolved).map(Resolution::Tests),
        }
    }

    fn resolved(&self, survey: &Survey) -> Result<Vec<Child>, Failure> {
        let primary = self.choices.iter().find(|choice| survey.file(choice.as_ref()).is_some());
        let candidates: Vec<&Child> = primary
            .into_iter()
            .chain(self.conditional.iter().filter(|child| survey.file(child.as_ref()).is_some()))
            .collect();
        let firsts: Vec<Option<usize>> = candidates
            .iter()
            .enumerate()
            .map(|(index, child)| candidates.iter().position(|other| other == child).filter(|first| *first == index))
            .collect();
        let found: Vec<Child> =
            candidates.into_iter().zip(firsts).filter_map(|(child, first)| first.map(|_| child.clone())).collect();
        found.first().map(|_| ()).map(|()| found).ok_or_else(|| Failure::Io {
            path: self.expected.clone(),
            source: io::Error::from(io::ErrorKind::NotFound),
        })
    }
}
