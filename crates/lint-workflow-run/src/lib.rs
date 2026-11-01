mod actions;
mod characters;
mod document;
mod finding;

use std::fs;
use std::path::Path;

use actions::Actions;
use document::Document;
use finding::Finding;
use lint_core::{Headline, RepoFile, Report, Site, Violation};
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct WorkflowRun;

impl WorkflowRun {
    const WORKFLOWS: &'static str = ".github/workflows";
    const EXTENSIONS: [&'static str; 2] = ["yml", "yaml"];

    #[must_use]
    pub fn report(files: &[RepoFile]) -> Report {
        let actions = Actions::among(files);
        Report::new(
            Headline::new("a workflow run step is a short command; logic lives in Rust"),
            files
                .iter()
                .filter(|file| Self::is_workflow(file) || Actions::is_manifest(file))
                .flat_map(|file| Self::violations(file, actions))
                .collect(),
        )
    }

    pub(crate) fn is_workflow(file: &RepoFile) -> bool {
        file.below(Path::new(Self::WORKFLOWS)).is_some()
            && file
                .shown()
                .extension()
                .is_some_and(|extension| Self::EXTENSIONS.iter().any(|known| extension == *known))
    }

    fn violations(file: &RepoFile, actions: Actions<'_>) -> Vec<Violation> {
        match fs::read_to_string(file.shown()) {
            Ok(text) => Self::violations_in(file, &text, actions),
            Err(source) => vec![Violation::new(Site::of_file(file.shown()), Finding::Unreadable { source })],
        }
    }

    fn violations_in(file: &RepoFile, text: &str, actions: Actions<'_>) -> Vec<Violation> {
        match Document::parse(text) {
            Ok(document) => document.findings(actions).into_iter().map(|found| found.violation(file.shown())).collect(),
            Err(error) => vec![Violation::new(
                Site::new(file.shown(), LineNumber::from(error.marker().line())),
                Finding::Unparsable { reason: error.info().to_owned() },
            )],
        }
    }
}
