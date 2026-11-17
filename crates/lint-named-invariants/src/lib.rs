mod panics;

use graph::Model;
use lint_core::{Headline, Report, Violation};
use lint_file::ModelLint;

use panics::{Named, Panics};

#[derive(Debug)]
pub struct UnnamedPanics;

#[derive(Debug)]
pub struct NamedInvariants;

impl ModelLint for UnnamedPanics {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "a panic in shipped code names the invariant it asserts: invariant!(holds, Invariant::X), or invariant!(broken Invariant::X) in an arm that crashes; panic!, unreachable!, todo!, unimplemented! and the assert macros are refused outside tests",
            ),
            model.uses().filter_map(Panics::refused).collect(),
        )
    }
}

impl ModelLint for NamedInvariants {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("report only, named invariants per crate: the invariant! assertions shipped code makes"),
            Named::per_crate(model.uses()).into_iter().map(Violation::from).collect(),
        )
    }
}
