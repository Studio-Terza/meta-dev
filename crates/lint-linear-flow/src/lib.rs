mod file_flow;
mod finding;
mod flows;
mod function_flow;
mod tally;
mod verdict;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_file::{KinLint, Reads, Swept, SweptLint};
use lint_sweep::{Finds, Member};

use file_flow::Collector;

use flows::Flows;

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

impl SweptLint for LinearBodies {
    const HEADLINE: Headline = Headline::new(
        "linear bodies: a body is let bindings and pipeline expressions, with no if, match or loop; the one non-linear form is a pure router, a body that is one match or if let",
    );
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Collector::watching(file))
    }
}

impl KinLint for PureRouters {
    const HEADLINE: Headline = Headline::new(
        "pure routers: a body that is one match or if let matches an enum, Option or Result without guards, and each arm is a single call or value",
    );
    const READS: Reads = Reads::Every;
    const MEMBER: Member = Rule::PureRouters.member();

    type Leader = Swept<LinearBodies>;
}

impl KinLint for PlainConditions {
    const HEADLINE: Headline = Headline::new(
        "no boolean operators: &&, || and ! stay out of bodies; a condition is named as an enum or a predicate type",
    );
    const READS: Reads = Reads::Every;
    const MEMBER: Member = Rule::PlainConditions.member();

    type Leader = Swept<LinearBodies>;
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
