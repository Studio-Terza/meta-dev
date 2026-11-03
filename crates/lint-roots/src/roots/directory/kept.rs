use std::iter;
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
            .filter(|entry| self.survey.kind(entry) == Kind::Directory)
            .filter(|entry| self.skipped(entry).is_none())
            .cloned()
            .map(Directory)
            .collect()
    }

    #[must_use]
    pub fn preorder(&self, directories: Vec<Directory>) -> Vec<Directory> {
        directories
            .into_iter()
            .flat_map(|directory| {
                let deeper = self.preorder(self.below(directory.as_ref()));
                iter::once(directory).chain(deeper)
            })
            .collect()
    }

    #[must_use]
    pub fn files(&self, directory: &Path) -> Vec<PathBuf> {
        self.survey
            .entries(directory)
            .iter()
            .flat_map(|entry| self.entry_files(entry, self.survey.kind(entry)))
            .collect()
    }

    fn entry_files(&self, entry: &Path, kind: Kind) -> Vec<PathBuf> {
        match kind {
            Kind::Directory => self.skipped(entry).map_or_else(|| self.files(entry), |_| Vec::new()),
            Kind::File | Kind::Link | Kind::Missing => vec![entry.to_path_buf()],
        }
    }

    fn skipped(&self, entry: &Path) -> Option<&Ignored> {
        self.ignored.iter().find(|rule| rule.applying_to(self.survey, entry).is_some())
    }
}
