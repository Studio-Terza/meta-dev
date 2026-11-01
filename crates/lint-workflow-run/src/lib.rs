mod characters;
mod finding;
mod run_step;

use std::fs;
use std::path::Path;

use finding::Finding;
use lint_core::{Headline, RepoFile, Report, Site, Violation};
use lint_layout::LineNumber;
use run_step::RunStep;

#[derive(Debug)]
pub struct WorkflowRun;

impl WorkflowRun {
    const WORKFLOWS: &'static str = ".github/workflows";
    const EXTENSIONS: [&'static str; 2] = ["yml", "yaml"];

    #[must_use]
    pub fn report(files: &[RepoFile]) -> Report {
        Report::new(
            Headline::new("a workflow run step is a short command; logic lives in Rust"),
            files.iter().filter(|file| Self::is_workflow(file)).flat_map(Self::violations).collect(),
        )
    }

    fn is_workflow(file: &RepoFile) -> bool {
        file.below(Path::new(Self::WORKFLOWS)).is_some()
            && file
                .shown()
                .extension()
                .is_some_and(|extension| Self::EXTENSIONS.iter().any(|known| extension == *known))
    }

    fn violations(file: &RepoFile) -> Vec<Violation> {
        match fs::read_to_string(file.shown()) {
            Ok(text) => {
                text.lines().enumerate().filter_map(|(index, line)| Self::violation(file, index, line)).collect()
            }
            Err(source) => vec![Violation::new(Site::of_file(file.shown()), Finding::Unreadable { source })],
        }
    }

    fn violation(file: &RepoFile, index: usize, line: &str) -> Option<Violation> {
        let finding = line.parse::<RunStep>().ok()?.finding()?;
        Some(Violation::new(Site::new(file.shown(), LineNumber::from(index + 1)), finding))
    }
}
