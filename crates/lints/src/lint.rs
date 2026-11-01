use lint_core::{Report, SourceFile};

pub trait Lint {
    fn report(files: &[SourceFile]) -> Report;
}
