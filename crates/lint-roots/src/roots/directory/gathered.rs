use std::path::{Path, PathBuf};

use file_ports::Kind;

use super::ignored::Ignored;
use crate::survey::Settled;

#[derive(Debug)]
pub struct Gathered<'survey> {
    survey: Settled<'survey>,
    ignored: &'survey [Ignored],
}

impl<'survey> Gathered<'survey> {
    #[must_use]
    pub fn new(survey: Settled<'survey>, ignored: &'survey [Ignored]) -> Gathered<'survey> {
        Gathered { survey, ignored }
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
            Kind::Directory => {
                Ignored::among(self.ignored, &self.survey, entry).map_or_else(|| self.files(entry), |_| Vec::new())
            }
            Kind::File | Kind::Link | Kind::Missing => vec![entry.to_path_buf()],
        }
    }
}
