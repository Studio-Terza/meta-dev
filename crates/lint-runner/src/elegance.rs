use lint_core::{Lint, Report, SourceFile};
use lint_repeated_bodies::RepeatedBodies;
use lint_unconsumed_traits::UnconsumedTraits;

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report> {
        vec![RepeatedBodies::report(files), UnconsumedTraits::report(files)]
    }
}
