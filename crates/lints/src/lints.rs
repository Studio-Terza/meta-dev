use std::path::Path;

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
        Ok(Outcome::from_reports(Self::reports(&SourceFiles::read(root)?)))
    }
}
