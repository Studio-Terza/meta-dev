use lint_core::{Lint, Report, SourceFile};

use crate::sources::{Lines, Sources};

pub trait Checks {
    fn reported<Shown>(files: &[SourceFile], shown: impl FnOnce(&Report<'_>) -> Shown) -> Shown;

    fn flagged(files: &[SourceFile]) -> Lines {
        Self::reported(files, Sources::lines)
    }
}

impl<Checked: Lint> Checks for Checked {
    fn reported<Shown>(files: &[SourceFile], shown: impl FnOnce(&Report<'_>) -> Shown) -> Shown {
        shown(&Checked::report(files))
    }
}
