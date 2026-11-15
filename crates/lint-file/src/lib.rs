mod concluded;
mod file_facts;

use std::fmt::Debug;
use std::marker::PhantomData;

use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_sweep::{Finds, Gathers, Judgement, Member, Sweep};

pub use concluded::{Concluded, Listed};
pub use file_facts::FileFacts;

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

pub trait KinLint {
    const HEADLINE: Headline;
    const READS: Reads;
    const MEMBER: Member;

    type Leader: SweptLint;
}

pub trait ContextLint {
    const HEADLINE: Headline;
    const READS: Reads;

    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Gathering<'head>: Gathers<Self::Part<'head>>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Self::Gathering<'head>;

    fn completed<'head>(part: Self::Part<'head>, _: &FileFacts<'head>) -> Self::Part<'head> {
        part
    }

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
pub struct Kin<Checked>(PhantomData<Checked>);

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

impl<Checked: KinLint> Lint for Kin<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        let found = read.flat_map(|file| Sweep::sorted_for(file, Checked::Leader::watcher(file), Checked::MEMBER));
        Report::new(Checked::HEADLINE, found.collect())
    }
}

impl<Checked: ContextLint> Lint for Judged<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let context: Checked::Context<'_> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                FileFacts::completed(file, parsed, Checked::gathering(parsed), Checked::completed)
            })
            .collect();
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

    fn primer_completed<'head>(part: Self::PrimerPart<'head>, _: &FileFacts<'head>) -> Self::PrimerPart<'head> {
        part
    }

    fn contribution<'run, 'head: 'run>(
        parsed: Parsed<'run, 'head>,
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
        let primer: Checked::Primer<'_> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                FileFacts::completed(file, parsed, Checked::priming(parsed), Checked::primer_completed)
            })
            .collect();
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

    type Gleaner<'run, 'head: 'run>: Gathers<Self::Part<'head>>;

    fn gleaner<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> Self::Gleaner<'run, 'head>;
}
