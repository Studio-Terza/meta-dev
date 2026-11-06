use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use lint_fixtures::{Sources, Text};
use lint_transition_graph::{Extraction, GraphLint, TypeSystem};

pub(crate) struct Shown;

impl Shown {
    pub(crate) fn by(pair: &[&str], reported: impl Fn(&[SourceFile]) -> Vec<String>) -> Vec<String> {
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

    pub(crate) fn of<Checked: Lint>(pair: &[&str]) -> Vec<String> {
        Self::by(pair, |files| Self::lines(&Checked::report(files)))
    }

    pub(crate) fn lines(report: &Report<'_>) -> Vec<String> {
        report.violations().iter().map(ToString::to_string).collect()
    }
}

pub(crate) struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let extraction = Extraction::from(files);
        Graph::report_graph(&TypeSystem::built_from(files, &extraction))
    }
}
