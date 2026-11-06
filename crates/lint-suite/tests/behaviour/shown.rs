use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use lint_fixtures::{Sources, Text};
use lint_transition_graph::{Extraction, GraphLint, TypeSystem};

#[derive(Debug, PartialEq)]
pub(crate) enum Presence {
    Present,
    Absent,
}

pub(crate) struct Shown;

impl Shown {
    pub(crate) fn by(pair: &[&str], reported: impl Fn(&[SourceFile]) -> String) -> Presence {
        <&[&str; 2]>::try_from(pair)
            .ok()
            .and_then(|[text, expected]| {
                reported(&[Sources::file("crates/x/src/lib.rs", Text::new(text))])
                    .matches(expected)
                    .next()
                    .map(|_| Presence::Present)
            })
            .unwrap_or(Presence::Absent)
    }

    pub(crate) fn of<Checked: Lint>(pair: &[&str]) -> Presence {
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
