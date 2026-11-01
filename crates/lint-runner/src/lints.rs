use std::path::{Path, PathBuf};

use lint_core::{Failure, Lint, Readout, Report, SourceFile, SourceFiles};

use lint_const_shape::ConstShape;
use lint_crate_graph::CrateGraph;
use lint_distinct_signatures::DistinctSignatures;
use lint_enum_predicates::EnumPredicates;
use lint_expression_flow::ExpressionFlow;
use lint_fn_shape::FnShape;
use lint_literal_names::LiteralNames;
use lint_manual_iteration::ManualIteration;
use lint_module_nesting::ModuleNesting;
use lint_named_lifetimes::NamedLifetimes;
use lint_named_numbers::NamedNumbers;
use lint_no_comments::NoComments;
use lint_no_forwarders::NoForwarders;
use lint_no_free_fns::NoFreeFns;
use lint_no_numbers_in_binaries::NoNumbersInBinaries;
use lint_no_parameter_bags::NoParameterBags;
use lint_primitive_boundary::PrimitiveBoundary;
use lint_private_fns::PrivateFns;
use lint_process_starters::ProcessStarters;
use lint_public_surface::PublicSurface;
use lint_repeated_methods::RepeatedMethods;
use lint_state_graph::StateGraph;
use lint_thin_main::ThinMain;
use lint_type_shape::TypeShape;
use lint_vec_out_params::VecOutParams;

use crate::Outcome;

#[derive(Debug)]
pub struct Lints;

impl Lints {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report> {
        vec![
            NoComments::report(files),
            NoFreeFns::report(files),
            ThinMain::report(files),
            VecOutParams::report(files),
            NamedLifetimes::report(files),
            EnumPredicates::report(files),
            ConstShape::report(files),
            TypeShape::report(files),
            DistinctSignatures::report(files),
            ExpressionFlow::report(files),
            FnShape::report(files),
            LiteralNames::report(files),
            ManualIteration::report(files),
            ModuleNesting::report(files),
            NamedNumbers::report(files),
            NoForwarders::report(files),
            NoNumbersInBinaries::report(files),
            NoParameterBags::report(files),
            PrimitiveBoundary::report(files),
            PrivateFns::report(files),
            ProcessStarters::report(files),
            PublicSurface::report(files),
            RepeatedMethods::report(files),
            StateGraph::report(files),
            CrateGraph::report(files),
        ]
    }

    pub fn check_root(root: &Path) -> Result<Outcome, Failure> {
        Self::check_all(&[root.to_path_buf()])
    }

    pub fn check_all(roots: &[PathBuf]) -> Result<Outcome, Failure> {
        let mut readout: Readout = SourceFiles::read_all(roots)?;
        let reports = Self::reports(readout.files());
        Ok(Outcome::from_reports(reports, readout.take_failures()))
    }
}
