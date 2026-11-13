mod file_flow;
mod finding;
mod flows;
mod function_flow;
mod tally;
mod verdict;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_file::{FileLint, Reads};

use flows::Flows;

use flows::FlowLint;

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

impl FlowLint for LinearBodies {
    const HEADLINE: &'static str = "linear bodies: a body is let bindings and pipeline expressions, with no if, match or loop; the one non-linear form is a pure router, a body that is one match or if let";
    const RULE: Rule = Rule::LinearBodies;
}

impl FileLint for LinearBodies {
    const HEADLINE: Headline = Headline::new(<Self as FlowLint>::HEADLINE);
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Flows::flagged::<Self>(file)
    }
}

impl FlowLint for PureRouters {
    const HEADLINE: &'static str = "pure routers: a body that is one match or if let matches an enum, Option or Result without guards, and each arm is a single call or value";
    const RULE: Rule = Rule::PureRouters;
}

impl FileLint for PureRouters {
    const HEADLINE: Headline = Headline::new(<Self as FlowLint>::HEADLINE);
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Flows::flagged::<Self>(file)
    }
}

impl FlowLint for PlainConditions {
    const HEADLINE: &'static str =
        "no boolean operators: &&, || and ! stay out of bodies; a condition is named as an enum or a predicate type";
    const RULE: Rule = Rule::PlainConditions;
}

impl FileLint for PlainConditions {
    const HEADLINE: Headline = Headline::new(<Self as FlowLint>::HEADLINE);
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Flows::flagged::<Self>(file)
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
