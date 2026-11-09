use std::marker::PhantomData;

use code_reader::Reader;
use lint_core::{Lint, ModelLint, SourceFile};

use crate::sources::{Lines, Sources};

pub trait Checks {
    fn flagged(files: &[SourceFile]) -> Lines;
}

#[derive(Debug)]
pub struct Queried<Checked>(PhantomData<Checked>);

impl<Checked: Lint> Checks for Checked {
    fn flagged(files: &[SourceFile]) -> Lines {
        Sources::lines(&Checked::report(files))
    }
}

impl<Checked: ModelLint> Checks for Queried<Checked> {
    fn flagged(files: &[SourceFile]) -> Lines {
        Sources::lines(&Checked::report(&Reader::read_files(files)))
    }
}
