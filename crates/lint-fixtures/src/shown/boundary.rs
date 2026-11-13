use std::str::FromStr;

use code_reader::Reader;
use lint_core::{FileHead, Parsed, Report, SourceFile, WorkspaceLint};
use newtype::Wrapping;
use workspace_metadata::Metadata;

use crate::{Checks, Source, Sources, Text};

#[derive(Debug)]
pub struct Shown;

impl Shown {
    #[must_use]
    pub fn filtered(pair: &[&str], reported: impl Fn(&[SourceFile]) -> Vec<String>) -> Vec<String> {
        <&[&str; 2]>::try_from(pair)
            .into_iter()
            .flat_map(|[text, subject]| {
                reported(&[Sources::file("crates/x/src/lib.rs", Text::new(text))])
                    .into_iter()
                    .filter(|line| line.contains(subject))
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    #[must_use]
    pub fn reported<Checked: Checks>(pair: &[&str]) -> Vec<String> {
        Self::filtered(pair, |files| Checked::reported(files, Self::listed))
    }

    #[must_use]
    pub fn placed_in<Checked: WorkspaceLint>(
        manifests: &[Source],
        sources: &[Source],
        workspace: Text<'_>,
    ) -> Vec<String> {
        let workspaces = [Sources::kept(Metadata::from_str(workspace.into_inner()))];
        let files = Sources::placed(manifests, sources);
        let parsed: Vec<Parsed<'_, '_>> = files.iter().map(Parsed::from).collect();
        let heads: Vec<FileHead> = files.iter().map(|file| file.head().clone()).collect();
        let model = Reader::read_files(&files);
        Self::listed(&Checked::report(&heads, &workspaces, &Checked::gathered(&parsed), &model))
    }

    #[must_use]
    pub fn listed(report: &Report<'_>) -> Vec<String> {
        report.violations().iter().map(ToString::to_string).collect()
    }
}
