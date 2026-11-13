use std::fmt;

use code_reader::Reader;
use lint_core::SourceFile;
use lint_transition_graph::{Extraction, Transitions, TypeSystem};
use workspace_metadata::Metadata;

use crate::query::Query;

#[derive(Debug)]
pub struct Answered(String);

impl Query {
    #[must_use]
    pub fn answered(&self, files: &[SourceFile], workspaces: &[Metadata]) -> Answered {
        let extraction = Extraction::from(files);
        let model = Reader::read_files(files);
        let transitions = Transitions::from(&model);
        let system = TypeSystem::built_from(&extraction, &transitions);
        let sized = self.sized(files, workspaces).map_or_default(|sizes| sizes.to_string());
        Answered(format!("{}{sized}", self.reply(&system)))
    }
}

impl fmt::Display for Answered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
