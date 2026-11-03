mod actions;
mod characters;
mod document;
mod excerpt;
mod finding;
mod word;

use std::path::Path;

use actions::Actions;
use document::Document;
use excerpt::Excerpt;
use finding::Finding;
use lint_core::{Headline, RepoFile, Report, Site, Violation};
use lint_layout::LineNumber;
use newtype::Newtype;
use saphyr::ScanError;

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
                .filter_map(|file| Self::workflow(file).or_else(|| Actions::manifest(file)))
                .flat_map(|file| Self::violations(file, actions))
                .collect(),
        )
    }

    pub(crate) fn workflow(file: &RepoFile) -> Option<&RepoFile> {
        file.below(Path::new(Self::WORKFLOWS))
            .and(file.shown().extension())
            .filter(|extension| Self::EXTENSIONS.iter().any(|known| *extension == *known))
            .map(|_| file)
    }

    fn violations(file: &RepoFile, actions: Actions<'_>) -> Vec<Violation> {
        let text = file
            .contents()
            .read()
            .map_err(Excerpt::from)
            .and_then(|bytes| String::from_utf8(bytes.clone().into_inner()).map_err(Excerpt::from));
        match text {
            Ok(text) => Self::violations_in(file, Document::try_from(text.as_str()), actions),
            Err(reason) => vec![Violation::new(Site::of_file(file.shown()), Finding::Unreadable { reason })],
        }
    }

    fn violations_in(file: &RepoFile, parsed: Result<Document<'_>, ScanError>, actions: Actions<'_>) -> Vec<Violation> {
        match parsed {
            Ok(document) => document.findings(actions).into_iter().map(|found| found.violation(file.shown())).collect(),
            Err(error) => vec![Violation::new(
                Site::new(file.shown(), LineNumber::from(error.marker().line())),
                Finding::Unparsable { reason: Excerpt::new(error.info().to_owned()) },
            )],
        }
    }
}
