mod baseline_names;
mod suppressions;

use baseline_names::BaselineNames;
use lint_core::{Headline, Lint, RepoFile, Report, Site, SourceFile, Violation};
use suppressions::Suppressions;

#[derive(Debug)]
pub struct NoSuppressions;

impl NoSuppressions {
    const ATTRIBUTE: &str = "lint switched off by an attribute; every lint is a lint for a reason, so fix the code \
                             and remove the attribute";
    const LIST: &str = "baseline or allow-list file; no lint has a baseline, an allow-list or a per-repository \
                        switch, so fix the findings and delete the file";

    #[must_use]
    pub fn baselines(files: &[RepoFile]) -> Report {
        Report::new(
            Headline::new("no baselines or allow-lists"),
            files
                .iter()
                .filter(|file| BaselineNames::listed(file.shown()))
                .map(|file| Violation::new(Site::of_file(file.shown()), Self::LIST))
                .collect(),
        )
    }
}

impl Lint for NoSuppressions {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("no lint is switched off in code"),
            files
                .iter()
                .flat_map(|file| {
                    Suppressions::lines_in(file.syntax())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), Self::ATTRIBUTE))
                })
                .collect(),
        )
    }
}
