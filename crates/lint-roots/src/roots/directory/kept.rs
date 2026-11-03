use std::path::{Path, PathBuf};

use file_ports::Kind;

use super::Directory;
use super::ignored::Ignored;
use crate::survey::Survey;

#[derive(Debug)]
pub struct Kept<'survey> {
    survey: &'survey Survey,
    ignored: &'survey [Ignored],
}

impl<'survey> Kept<'survey> {
    #[must_use]
    pub fn new(survey: &'survey Survey, ignored: &'survey [Ignored]) -> Kept<'survey> {
        Kept { survey, ignored }
    }

    #[must_use]
    pub fn below(&self, directory: &Path) -> Vec<Directory> {
        self.survey
            .entries(directory)
            .iter()
            .filter(|entry| self.survey.kind(entry) == Kind::Directory && self.skipped(entry).is_none())
            .cloned()
            .map(Directory)
            .collect()
    }

    #[must_use]
    pub fn preorder(&self, directory: Directory) -> Vec<Directory> {
        let below = self.below(directory.as_ref());
        [directory].into_iter().chain(below.into_iter().flat_map(|subdirectory| self.preorder(subdirectory))).collect()
    }

    #[must_use]
    pub fn files(&self, directory: &Path) -> Vec<PathBuf> {
        self.survey
            .entries(directory)
            .iter()
            .flat_map(|entry| match (self.survey.kind(entry), self.skipped(entry)) {
                (Kind::Directory, Some(_)) => Vec::new(),
                (Kind::Directory, None) => self.files(entry),
                (Kind::File | Kind::Link | Kind::Missing, _) => vec![entry.clone()],
            })
            .collect()
    }

    fn skipped(&self, entry: &Path) -> Option<&Ignored> {
        self.ignored.iter().find(|rule| rule.applying_to(self.survey, entry).is_some())
    }
}
