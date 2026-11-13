mod concluded;

use std::fmt::Debug;
use std::marker::PhantomData;

use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_sweep::{Finds, Gathers, Judgement, Sweep};

pub use concluded::{Concluded, Listed};

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

pub trait SweptLint {
    const HEADLINE: Headline;
    const READS: Reads;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_>;
}

pub trait ContextLint {
    const HEADLINE: Headline;
    const READS: Reads;

    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Gathering<'head>: Gathers<Self::Part<'head>>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Self::Gathering<'head>;

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        context: &'run Self::Context<'head>,
    ) -> Judgement<'file, 'run>;
}

#[derive(Debug)]
pub struct Filed<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Swept<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Judged<Checked>(PhantomData<Checked>);

impl<Checked: FileLint> Lint for Filed<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(Checked::violations).collect())
    }
}

impl<Checked: SweptLint> Lint for Swept<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(|file| Sweep::found(file, Checked::watcher(file))).collect())
    }
}

impl<Checked: ContextLint> Lint for Judged<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let context: Checked::Context<'_> =
            files.iter().map(|file| Sweep::gathered(file, Checked::gathering(Parsed::from(file)))).collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(
            Checked::HEADLINE,
            read.flat_map(|file| Checked::judgement(file, &context).violations(file)).collect(),
        )
    }
}

pub trait PrimedFacts {
    type PrimerPart<'head>: Send + Debug;
    type Primer<'head>: FromIterator<Self::PrimerPart<'head>> + Send + Sync + Debug;
    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Priming<'head>: Gathers<Self::PrimerPart<'head>>;
    type Contribution<'run, 'head: 'run>: Gathers<Self::Part<'head>>;

    fn priming<'head>(parsed: Parsed<'_, 'head>) -> Self::Priming<'head>;

    fn contribution<'run, 'head: 'run>(
        parsed: Parsed<'_, 'head>,
        primer: &'run Self::Primer<'head>,
    ) -> Self::Contribution<'run, 'head>;
}

pub trait PrimedLint: PrimedFacts {
    const HEADLINE: Headline;
    const READS: Reads;

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        primer: &'run Self::Primer<'head>,
        context: &'run Self::Context<'head>,
    ) -> Judgement<'file, 'run>;
}

#[derive(Debug)]
pub struct Primed<Checked>(PhantomData<Checked>);

impl<Checked: PrimedLint> Lint for Primed<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let primer: Checked::Primer<'_> =
            files.iter().map(|file| Sweep::gathered(file, Checked::priming(Parsed::from(file)))).collect();
        let context: Checked::Context<'_> = files
            .iter()
            .map(|file| Sweep::gathered(file, Checked::contribution(Parsed::from(file), &primer)))
            .collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(
            Checked::HEADLINE,
            read.flat_map(|file| Checked::judgement(file, &primer, &context).violations(file)).collect(),
        )
    }
}

pub trait Gleaning {
    type Part<'head>: Send + Debug;
    type Gleaned<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Gleaner<'head>: Gathers<Self::Part<'head>>;

    fn gleaner<'head>(parsed: Parsed<'_, 'head>) -> Self::Gleaner<'head>;
}
