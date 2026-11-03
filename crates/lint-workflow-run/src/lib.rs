mod actions;
mod characters;
mod document;
mod excerpt;
mod finding;
mod judged;
mod word;

use actions::Actions;
use document::Document;
use excerpt::Excerpt;
use finding::Finding;
use judged::Judged;
use lint_core::{Headline, Lint, RepoFile, Report, Site, Violation};
use lint_layout::LineNumber;
use newtype::Newtype;
use saphyr::ScanError;

#[derive(Debug)]
pub struct WorkflowRun;

impl WorkflowRun {
    fn violations<'file>(file: &'file RepoFile, actions: Actions<'_>) -> Vec<Violation<'file>> {
        let text = file
            .contents()
            .read()
            .map_err(Excerpt::from)
            .and_then(|bytes| String::from_utf8(bytes.clone().into_inner()).map_err(Excerpt::from));
        text.map_or_else(
            |reason| vec![Violation::new(Site::of_file(file.shown()), Finding::Unreadable { reason })],
            |text| Self::violations_in(file, Document::try_from(text.as_str()), actions),
        )
    }

    fn violations_in<'file>(
        file: &'file RepoFile,
        parsed: Result<Document<'_>, ScanError>,
        actions: Actions<'_>,
    ) -> Vec<Violation<'file>> {
        match parsed {
            Ok(document) => document.findings(actions).into_iter().map(|found| found.violation(file.shown())).collect(),
            Err(error) => vec![Violation::new(
                Site::new(file.shown(), LineNumber::from(error.marker().line())),
                Finding::Unparsable { reason: Excerpt::new(error.info().to_owned()) },
            )],
        }
    }
}

impl Lint<RepoFile> for WorkflowRun {
    fn report(files: &[RepoFile]) -> Report<'_> {
        let actions = Actions::among(files);
        Report::new(
            Headline::new("a workflow run step is a short command; logic lives in Rust"),
            files
                .iter()
                .filter(|file| Judged::judging(file).is_some())
                .flat_map(|file| Self::violations(file, actions))
                .collect(),
        )
    }
}
