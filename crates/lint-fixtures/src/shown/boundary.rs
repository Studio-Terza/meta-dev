use lint_core::{Lint, Report, SourceFile};

use crate::{Sources, Text};

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
    pub fn reported<Checked: Lint>(pair: &[&str]) -> Vec<String> {
        Self::filtered(pair, |files| Self::listed(&Checked::report(files)))
    }

    #[must_use]
    pub fn listed(report: &Report<'_>) -> Vec<String> {
        report.violations().iter().map(ToString::to_string).collect()
    }
}
