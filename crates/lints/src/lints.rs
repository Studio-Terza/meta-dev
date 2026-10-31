use std::path::{Path, PathBuf};

use lint_core::{Failure, Readout, Report, SourceFile, SourceFiles};

use crate::{
    ConstShape, DistinctSignatures, EnumPredicates, ExpressionFlow, FnShape, Lint, LiteralNames, ManualIteration,
    ModuleNesting, NamedLifetimes, NamedNumbers, NoComments, NoForwarders, NoFreeFns, NoNumbersInBinaries,
    NoParameterBags, Outcome, PrimitiveBoundary, PrivateFns, ProcessStarters, PublicSurface, RepeatedMethods,
    StateGraph, ThinMain, TypeShape, VecOutParams,
};

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
