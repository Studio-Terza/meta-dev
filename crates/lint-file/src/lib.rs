mod lead;

use std::fmt::Debug;
use std::marker::PhantomData;

use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_sweep::{Finds, Gathers, Sweep};

pub use lead::{Lead, Leader, Start};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reads {
    Shipped,
    Every,
}

impl Reads {
    #[must_use]
    pub fn admits(self, file: &SourceFile) -> Option<&SourceFile> {
        match self {
            Reads::Shipped => file.head().shipped().map(|_| file),
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
}

pub trait WatchingLint: ContextLint {
    type Found<'file>;
    type Watched<'file>: Gathers<Self::Found<'file>> + Debug;

    fn watched(file: &SourceFile) -> Self::Watched<'_>;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Self::Found<'file>,
        context: &Self::Context<'head>,
    ) -> Vec<Violation<'file>>;
}

pub trait RuledLint: ContextLint {
    fn ruled<'file, 'head: 'file>(file: &'file SourceFile, context: &Self::Context<'head>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
pub struct Filed<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Swept<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Judged<Checked>(PhantomData<Checked>);

#[derive(Debug)]
pub struct Ruled<Checked>(PhantomData<Checked>);

impl<Checked: FileLint + 'static> Leader for Filed<Checked> {
    const LEAD: Lead = Lead::Filed(Checked::violations);
}

impl<Checked: SweptLint + 'static> Leader for Swept<Checked> {
    const LEAD: Lead = Lead::Swept(Checked::watcher);
}

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

impl<Checked: RuledLint> Lint for Ruled<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let context: Checked::Context<'_> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                Sweep::gathered(file, Checked::gathering(parsed))
            })
            .collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(|file| Checked::ruled(file, &context)).collect())
    }
}

impl<Checked: WatchingLint> Lint for Judged<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let walked: Vec<Walked<Checked::Part<'_>, Checked::Found<'_>>> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                let watched = Checked::READS.admits(file).map(Checked::watched);
                let walking = Walking { gathering: Checked::gathering(parsed), watched };
                Sweep::gathered(file, walking)
            })
            .collect();
        let (parts, found): (Vec<Checked::Part<'_>>, Vec<Option<Checked::Found<'_>>>) =
            walked.into_iter().map(|walk| (walk.part, walk.found)).unzip();
        let context: Checked::Context<'_> = parts.into_iter().collect();
        let judged = files.iter().zip(found).flat_map(|(file, watched)| {
            watched.map(|seen| Checked::judged(file, seen, &context)).into_iter().flatten()
        });
        Report::new(Checked::HEADLINE, judged.collect())
    }
}

#[derive(Debug)]
struct Walked<Part, Found> {
    part: Part,
    found: Option<Found>,
}

#[derive(Debug)]
struct Walking<Gathering, Watched> {
    gathering: Gathering,
    watched: Option<Watched>,
}

impl<Part, Found, Gathering: Gathers<Part>, Watched: Gathers<Found>> Gathers<Walked<Part, Found>>
    for Walking<Gathering, Watched>
{
    fn gathered(self, file: &SourceFile) -> Walked<Part, Found> {
        Walked { part: self.gathering.gathered(file), found: self.watched.map(|watched| watched.gathered(file)) }
    }
}
