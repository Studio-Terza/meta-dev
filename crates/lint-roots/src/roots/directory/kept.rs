use std::iter;
use std::path::Path;

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
    pub const fn new(survey: &'survey Survey, ignored: &'survey [Ignored]) -> Self {
        Kept { survey, ignored }
    }

    #[must_use]
    pub fn below(&self, directory: &Path) -> Vec<Directory> {
        self.survey
            .entries(directory)
            .iter()
            .filter(|entry| self.survey.kind(entry) == Kind::Directory)
            .filter(|entry| Ignored::among(self.ignored, self.survey, entry).is_none())
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
}
