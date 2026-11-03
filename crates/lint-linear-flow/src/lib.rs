mod file_flow;
mod finding;
mod function_flow;
mod tally;
mod verdict;

use lint_core::{Headline, Lint, Report, SourceFile};

use file_flow::FileFlow;
use finding::Rule;
use tally::Tally;

#[derive(Debug)]
pub struct LinearBodies;

#[derive(Debug)]
pub struct PureRouters;

#[derive(Debug)]
pub struct PlainConditions;

#[derive(Debug)]
pub struct FlowShapes;

impl Lint for LinearBodies {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "linear bodies: a body is let bindings and pipeline expressions, with no if, match or loop; the one non-linear form is a pure router, a body that is one match or if let",
            ),
            FileFlow::violations(&FileFlow::of_files(files), Rule::LinearBodies),
        )
    }
}

impl Lint for PureRouters {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "pure routers: a body that is one match or if let matches an enum, Option or Result without guards, and each arm is a single call or value",
            ),
            FileFlow::violations(&FileFlow::of_files(files), Rule::PureRouters),
        )
    }
}

impl Lint for PlainConditions {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "no boolean operators: &&, || and ! stay out of bodies; a condition is named as an enum or a predicate type",
            ),
            FileFlow::violations(&FileFlow::of_files(files), Rule::PlainConditions),
        )
    }
}

impl Lint for FlowShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, linear flow per crate: functions that are a pipeline, a pure router, or neither",
            ),
            Tally::of_flows(&FileFlow::of_files(files)),
        )
    }
}
