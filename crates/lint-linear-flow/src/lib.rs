mod file_flow;
mod flows;
mod function_flow;
mod rule;
mod tally;
mod verdict;

use lint_core::{Headline, Lint, Report, SourceFile};

pub use flows::Flows;

use flows::FlowLint;

use rule::Rule;
use tally::Tally;

#[derive(Debug)]
pub struct LinearBodies;

#[derive(Debug)]
pub struct PureRouters;

#[derive(Debug)]
pub struct PlainConditions;

#[derive(Debug)]
pub struct FlowShapes;

impl FlowLint for LinearBodies {
    const HEADLINE: &'static str = "linear bodies: a body is let bindings and pipeline expressions, with no if, match or loop; the one non-linear form is a pure router, a body that is one match or if let";
    const RULE: Rule = Rule::LinearBodies;
}

impl Lint for LinearBodies {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Flows::from(files).report::<Self>()
    }
}

impl FlowLint for PureRouters {
    const HEADLINE: &'static str = "pure routers: a body that is one match or if let matches an enum, Option or Result without guards, and each arm is a single call or value";
    const RULE: Rule = Rule::PureRouters;
}

impl Lint for PureRouters {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Flows::from(files).report::<Self>()
    }
}

impl FlowLint for PlainConditions {
    const HEADLINE: &'static str =
        "no boolean operators: &&, || and ! stay out of bodies; a condition is named as an enum or a predicate type";
    const RULE: Rule = Rule::PlainConditions;
}

impl Lint for PlainConditions {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Flows::from(files).report::<Self>()
    }
}

impl Lint for FlowShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, linear flow per crate: functions that are a pipeline, a pure router, or neither",
            ),
            Tally::of_flows(Flows::from(files).as_ref()),
        )
    }
}
