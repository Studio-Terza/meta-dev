mod concluded;

use std::fmt::Debug;
use std::marker::PhantomData;

use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};

pub use concluded::Concluded;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reads {
    Shipped,
    Every,
}

impl Reads {
    #[must_use]
    pub fn admits(self, file: &SourceFile) -> Option<&SourceFile> {
        match self {
            Reads::Shipped => file.shipped(),
            Reads::Every => Some(file),
        }
    }
}

pub trait FileLint {
    const HEADLINE: Headline;
    const READS: Reads;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>>;
}

pub trait ContextLint {
    const HEADLINE: Headline;
    const READS: Reads;

    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    fn contributed<'head>(parsed: Parsed<'_, 'head>) -> Self::Part<'head>;

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, context: &Self::Context<'head>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
pub struct Filed<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Judged<Checked>(PhantomData<Checked>);

impl<Checked: FileLint> Lint for Filed<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(Checked::violations).collect())
    }
}

impl<Checked: ContextLint> Lint for Judged<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let context: Checked::Context<'_> = files.iter().map(Parsed::from).map(Checked::contributed).collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(|file| Checked::judged(file, &context)).collect())
    }
}

pub trait PrimedLint {
    const HEADLINE: Headline;
    const READS: Reads;

    type PrimerPart<'head>: Send + Debug;
    type Primer<'head>: FromIterator<Self::PrimerPart<'head>> + Send + Sync + Debug;
    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> Self::PrimerPart<'head>;

    fn contributed<'head>(parsed: Parsed<'_, 'head>, primer: &Self::Primer<'head>) -> Self::Part<'head>;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        primer: &Self::Primer<'head>,
        context: &Self::Context<'head>,
    ) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
pub struct Primed<Checked>(PhantomData<Checked>);

impl<Checked: PrimedLint> Lint for Primed<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let primer: Checked::Primer<'_> = files.iter().map(Parsed::from).map(Checked::primed).collect();
        let context: Checked::Context<'_> =
            files.iter().map(Parsed::from).map(|parsed| Checked::contributed(parsed, &primer)).collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(|file| Checked::judged(file, &primer, &context)).collect())
    }
}
