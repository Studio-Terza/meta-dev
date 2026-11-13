use std::marker::PhantomData;

use code_reader::Reader;
use lint_core::{Lint, ModelLint, Report, SourceFile};

use crate::sources::{Lines, Sources};

pub trait Checks {
    fn reported<Shown>(files: &[SourceFile], shown: impl FnOnce(&Report<'_>) -> Shown) -> Shown;

    fn flagged(files: &[SourceFile]) -> Lines {
        Self::reported(files, Sources::lines)
    }
}

#[derive(Debug)]
pub struct Queried<Checked>(PhantomData<Checked>);

impl<Checked: Lint> Checks for Checked {
    fn reported<Shown>(files: &[SourceFile], shown: impl FnOnce(&Report<'_>) -> Shown) -> Shown {
        shown(&Checked::report(files))
    }
}

impl<Checked: ModelLint> Checks for Queried<Checked> {
    fn reported<Shown>(files: &[SourceFile], shown: impl FnOnce(&Report<'_>) -> Shown) -> Shown {
        shown(&Checked::report(&Reader::read_files(files)))
    }
}
