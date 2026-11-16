mod concluded;
mod file_facts;
mod lead;
mod one_walk;

use std::fmt::Debug;
use std::marker::PhantomData;

use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_sweep::{Finds, Gathers, Member, Sweep, Watching};

pub use concluded::{Concluded, Listed};
pub use file_facts::FileFacts;
pub use lead::{Lead, Leader, SortStart, Start};
pub use one_walk::OneWalk;

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

    type Leader: Leader;
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
pub struct Kin<Checked>(PhantomData<Checked>);

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

impl<Checked: KinLint> Lint for Kin<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        let found = read.flat_map(|file| Checked::Leader::LEAD.member_of(file, Checked::MEMBER));
        Report::new(Checked::HEADLINE, found.collect())
    }
}

impl<Checked: RuledLint> Lint for Ruled<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let context: Checked::Context<'_> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                FileFacts::completed(file, parsed, Checked::gathering(parsed), Checked::completed)
            })
            .collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        Report::new(Checked::HEADLINE, read.flat_map(|file| Checked::ruled(file, &context)).collect())
    }
}

impl<Checked: WatchingLint> Lint for Judged<Checked> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let walked: Vec<Done<Checked::Part<'_>, Checked::Found<'_>>> = files
            .iter()
            .map(|file| {
                let parsed = Parsed::from(file);
                let watched = Checked::READS.admits(file).map(Checked::watched);
                let walking =
                    Walking { gathering: Checked::gathering(parsed), facts: FileFacts::from(parsed), watched };
                let walked = Sweep::gathered(file, walking);
                Done { part: Checked::completed(walked.part, &walked.facts), found: walked.found }
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
struct Done<Part, Found> {
    part: Part,
    found: Option<Found>,
}

#[derive(Debug)]
struct Walked<'head, Part, Found> {
    part: Part,
    facts: FileFacts<'head>,
    found: Option<Found>,
}

#[derive(Debug)]
struct Walking<'head, Gathering, Watched> {
    gathering: Gathering,
    facts: FileFacts<'head>,
    watched: Option<Watched>,
}

impl<'head, Part, Found, Gathering: Gathers<Part>, Watched: Gathers<Found>> Gathers<Walked<'head, Part, Found>>
    for Walking<'head, Gathering, Watched>
{
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        let watched = self.watched.as_mut().map_or_default(Gathers::watching);
        self.gathering.watching().into_iter().chain(self.facts.watching()).chain(watched).collect()
    }

    fn gathered(self) -> Walked<'head, Part, Found> {
        Walked { part: self.gathering.gathered(), facts: self.facts, found: self.watched.map(Gathers::gathered) }
    }
}

pub trait PrimedFacts {
    type PrimerPart<'head>: Send + Debug;
    type Primer<'head>: FromIterator<Self::PrimerPart<'head>> + Send + Sync + Debug;
    type Part<'head>: Send + Debug;
    type Context<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Priming<'head>: Gathers<Self::PrimerPart<'head>>;
    type Raw<'head>: Send + Debug;
    type Recording<'run, 'head: 'run>: Gathers<Self::Raw<'head>>;

    fn priming<'head>(parsed: Parsed<'_, 'head>) -> Self::Priming<'head>;

    fn primer_completed<'head>(part: Self::PrimerPart<'head>, _: &FileFacts<'head>) -> Self::PrimerPart<'head> {
        part
    }

    fn recording<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> Self::Recording<'run, 'head>;

    fn contribution<'head>(raw: Self::Raw<'head>, primer: &Self::Primer<'head>) -> Self::Part<'head>;
}

pub trait PrimedLint: PrimedFacts {
    const HEADLINE: Headline;
    const READS: Reads;

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
        let mut walked: Vec<PrimedWalk<'_, Checked>> = files.iter().map(PrimedWalk::walked).collect();
        let primer: Checked::Primer<'_> = walked.iter_mut().filter_map(|walk| walk.primer.take()).collect();
        let context: Checked::Context<'_> = walked
            .iter_mut()
            .filter_map(|walk| walk.raw.take())
            .map(|raw| Checked::contribution(raw, &primer))
            .collect();
        let read = files.iter().filter_map(|file| Checked::READS.admits(file));
        let judged = read.flat_map(|file| Checked::judged(file, &primer, &context));
        Report::new(Checked::HEADLINE, judged.collect())
    }
}

struct PrimedWalk<'file, Checked: PrimedLint> {
    primer: Option<Checked::PrimerPart<'file>>,
    raw: Option<Checked::Raw<'file>>,
}

impl<'file, Checked: PrimedLint> PrimedWalk<'file, Checked> {
    fn walked(file: &'file SourceFile) -> PrimedWalk<'file, Checked> {
        let parsed = Parsed::from(file);
        let gathering = Two { first: Checked::priming(parsed), second: Checked::recording(parsed) };
        let walked = FileFacts::completed(file, parsed, gathering, |Two { first, second }, facts| Two {
            first: Checked::primer_completed(first, facts),
            second,
        });
        PrimedWalk { primer: Some(walked.first), raw: Some(walked.second) }
    }
}

#[derive(Debug)]
struct Two<First, Second> {
    first: First,
    second: Second,
}

impl<FirstPart, SecondPart, First: Gathers<FirstPart>, Second: Gathers<SecondPart>> Gathers<Two<FirstPart, SecondPart>>
    for Two<First, Second>
{
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.first.watching().into_iter().chain(self.second.watching()).collect()
    }

    fn gathered(self) -> Two<FirstPart, SecondPart> {
        Two { first: self.first.gathered(), second: self.second.gathered() }
    }
}

pub trait Gleaning {
    type Part<'head>: Send + Debug;
    type Gleaned<'head>: FromIterator<Self::Part<'head>> + Send + Sync + Debug;

    type Gleaner<'run, 'head: 'run>: Gathers<Self::Part<'head>>;

    fn gleaner<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> Self::Gleaner<'run, 'head>;
}
