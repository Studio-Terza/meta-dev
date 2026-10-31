use std::path::{Path, PathBuf};

use lint_core::{Failure, Report, SourceFile, SourceFiles};

use crate::{
    ConstShape, EnumPredicates, NamedLifetimes, NoComments, NoFreeFns, Outcome, ThinMain, TypeShape, VecOutParams,
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
        ]
    }

    pub fn check(root: &Path) -> Result<Outcome, Failure> {
        Self::check_all(&[root.to_path_buf()])
    }

    pub fn check_all(roots: &[PathBuf]) -> Result<Outcome, Failure> {
        let mut readout = SourceFiles::read_all(roots)?;
        let reports = Self::reports(readout.files());
        Ok(Outcome::from_reports(reports, readout.take_failures()))
    }
}
