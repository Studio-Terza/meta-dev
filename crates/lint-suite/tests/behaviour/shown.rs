use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use lint_fixtures::{Sources, Text};
use lint_transition_graph::{Extraction, GraphLint, TypeSystem};

pub(crate) struct Shown;

impl Shown {
    pub(crate) fn by(pair: &[&str], reported: impl Fn(&[SourceFile]) -> String) -> Vec<String> {
        <&[&str; 2]>::try_from(pair)
            .into_iter()
            .flat_map(|[text, expected]| {
                reported(&[Sources::file("crates/x/src/lib.rs", Text::new(text))])
                    .matches(expected)
                    .take(1)
                    .map(str::to_owned)
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    pub(crate) fn of<Checked: Lint>(pair: &[&str]) -> Vec<String> {
        Self::by(pair, |files| Checked::report(files).to_string())
    }
}

pub(crate) struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let extraction = Extraction::from(files);
        Graph::report_graph(&TypeSystem::built_from(files, &extraction))
    }
}
