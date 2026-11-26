use std::any::TypeId;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::thread::Scope;

use cargo_ports::Cargo;

use file_ports::Files;
use lint_core::{Headline, Readout, SourceFile, Violation};
use lint_file::{FileLint, Filed, Judged, Lead, Leader, Reads, Ruled, RuledLint, Start, Swept, SweptLint, WatchingLint};
use lint_reader::{Described, Unlocated};
use lint_sweep::{Finds, Found, Gathers};
use timer_ports::Timer;

use lint_bool_methods::BoolMethods;
use lint_const_shape::ConstShape;
use lint_enum_predicates::EnumPredicates;
use lint_expression_flow::ExpressionFlow;
use lint_fn_shape::FnShape;
use lint_literal_names::LiteralNames;
use lint_manual_iteration::ManualIteration;
use lint_named_lifetimes::NamedLifetimes;
use lint_named_numbers::NamedNumbers;
use lint_no_comments::NoComments;
use lint_no_forwarders::NoForwarders;
use lint_no_free_fns::NoFreeFns;
use lint_no_ignored_tests::NoIgnoredTests;
use lint_no_numbers_in_binaries::NoNumbersInBinaries;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_private_fns::PrivateFns;
use lint_process_starters::ProcessStarters;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_thin_main::ThinMain;
use lint_type_shape::TypeShape;
use lint_vec_out_params::VecOutParams;

use crate::cli::boundary::{FileOrder, Helper, Helpers, Rendered, Slot, Worked, Workers};
use crate::cli::ignored::Ignored;
use crate::cli::sources::{Claimed, Holding, Settled};
use crate::cli::timings::{Measured, Step, Stopwatch, Worker};
use crate::cli::{Outcome, Sources, Verdict};
use invariants::Ignoring;

#[derive(Debug)]
pub struct Lints;

#[derive(Debug)]
pub(crate) struct Crew<'scope> {
    helpers: Helpers<'scope>,
}

impl Crew<'_> {
    pub(crate) fn gated(
        self,
        readout: &Measured<Readout>,
        sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        let mine = Lints::worked(readout.done(), sources, Worker::MAIN);
        let worked: Worked = [mine].into_iter().chain(self.helpers.into_iter().map(Helper::joined)).collect();
        let first = sources.earlier().chain([readout.timing()]);
        let table = sources.wired().table(first.chain(worked.timings().iter().copied()), worked.visited().clone());
        let outcome = Lints::gathered(readout.done(), sources.located().done(), worked);
        outcome.surveyed(&[table], out, err)
    }
}

type Pend = for<'file> fn(&'file SourceFile) -> Box<dyn Awaits<'file> + 'file>;

#[derive(Clone, Copy, Debug)]
struct Judging {
    headline: Headline,
    reads: Reads,
    pend: Pend,
}

type Conclude = for<'file> fn(&'file SourceFile) -> Box<dyn Concludes<'file> + 'file>;

#[derive(Clone, Copy, Debug)]
struct Rules {
    headline: Headline,
    reads: Reads,
    conclude: Conclude,
}

#[derive(Clone, Copy, Debug)]
struct Filing {
    headline: Headline,
    reads: Reads,
    source: Source,
}

#[derive(Clone, Copy, Debug)]
struct Source {
    key: fn() -> TypeId,
    lead: Lead,
}

impl Lints {
    const LINTS: [Entry; 22] = [
        <Filed<NoComments> as Registered>::ENTRY,
        <Swept<NoFreeFns> as Registered>::ENTRY,
        <Filed<ThinMain> as Registered>::ENTRY,
        <Swept<VecOutParams> as Registered>::ENTRY,
        <Swept<NamedLifetimes> as Registered>::ENTRY,
        <Judged<EnumPredicates> as Registered>::ENTRY,
        <Swept<ConstShape> as Registered>::ENTRY,
        <Swept<TypeShape> as Registered>::ENTRY,
        <Swept<ExpressionFlow> as Registered>::ENTRY,
        <Swept<FnShape> as Registered>::ENTRY,
        <Swept<LiteralNames> as Registered>::ENTRY,
        <Swept<ManualIteration> as Registered>::ENTRY,
        <Judged<NamedNumbers> as Registered>::ENTRY,
        <Judged<NoForwarders> as Registered>::ENTRY,
        <Swept<NoNumbersInBinaries> as Registered>::ENTRY,
        <Swept<PrivateFns> as Registered>::ENTRY,
        <Filed<ProcessStarters> as Registered>::ENTRY,
        <Ruled<PublicSurface> as Registered>::ENTRY,
        <Judged<PublicApiTests> as Registered>::ENTRY,
        <Judged<NoTestOnlyApi> as Registered>::ENTRY,
        <Swept<NoIgnoredTests> as Registered>::ENTRY,
        <Swept<BoolMethods> as Registered>::ENTRY,
    ];

    pub(crate) fn started<'scope, 'run: 'scope, F: Files + Sync, C: Cargo + Sync, T: Timer + Sync>(
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, C, T>,
        readers: Vec<lint_reader::Reader<'run>>,
    ) -> Crew<'scope> {
        let helpers = Workers::available().helpers().zip(readers).filter_map(|(index, reader)| {
            Helper::spawned(scope, move || Self::helped(sources, Worker::from(index), reader))
                .ignoring(Ignored::Unspawned)
        });
        let locating = Helper::spawned(scope, move || Self::locating(sources)).ignoring(Ignored::Unspawned);
        Crew { helpers: locating.into_iter().chain(helpers).collect() }
    }

    fn gathered<'run>(
        readout: &'run Readout,
        located: &'run Result<Described, Unlocated>,
        worked: Worked,
    ) -> Outcome<'run> {
        let mut rendered = worked.into_rendered(|slot| slot.picked(&Self::LINTS).map(Entry::headline));
        rendered.sort_by_key(Rendered::slot);
        Outcome::located(located, rendered, readout.failures())
    }

    fn locating(sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Worked {
        Worked::timed(sources.located().timing())
    }

    fn helped(
        sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>,
        worker: Worker,
        reader: lint_reader::Reader<'_>,
    ) -> Worked {
        let readout = ManuallyDrop::new(sources.read(worker, reader));
        [Self::worked(readout.done(), sources, worker), Worked::timed(readout.timing())].into_iter().collect()
    }

    fn worked(
        readout: &Readout,
        sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>,
        worker: Worker,
    ) -> Worked {
        let mut lowered = sources.gathered(readout, worker, Filer::started);
        let mut work: Vec<FileWork<'_>> = lowered.filed();
        let filed: Worked = work.iter_mut().map(FileWork::worked).collect();
        let waiting = Stopwatch::started(sources.wired().timer());
        let settled = Settled::from(sources.gathering());
        let waited = waiting.stopped(Step::Judged(worker).waited());
        let judging = Stopwatch::started(sources.wired().timer());
        let judged: Worked = work.into_iter().map(|file| file.judged(settled)).collect();
        let judged_timing = judging.stopped(Step::Judged(worker));
        let timings = lowered.timings().iter().copied().chain([waited, judged_timing]);
        timings.map(Worked::timed).chain([filed, judged]).collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    work: Work,
}

#[derive(Clone, Copy, Debug)]
enum Work {
    Filed(Filing),
    Judged(Judging),
    Ruled(Rules),
}

impl Entry {
    fn filing(self) -> Option<Filing> {
        match self.work {
            Work::Filed(filing) => Some(filing),
            Work::Judged(_) | Work::Ruled(_) => None,
        }
    }

    fn judging(self) -> Option<Judging> {
        match self.work {
            Work::Judged(judging) => Some(judging),
            Work::Filed(_) | Work::Ruled(_) => None,
        }
    }

    fn ruling(self) -> Option<Rules> {
        match self.work {
            Work::Ruled(rules) => Some(rules),
            Work::Filed(_) | Work::Judged(_) => None,
        }
    }

    fn headline(&self) -> Headline {
        match self.work {
            Work::Filed(filing) => filing.headline,
            Work::Judged(judging) => judging.headline,
            Work::Ruled(rules) => rules.headline,
        }
    }
}

#[derive(Debug)]
struct Ruling<'file> {
    slot: Slot,
    pending: Box<dyn Awaits<'file> + 'file>,
}

#[derive(Debug)]
struct Concluding<'file> {
    slot: Slot,
    pending: Box<dyn Concludes<'file> + 'file>,
}

#[derive(Debug)]
pub(crate) struct FileWork<'file> {
    order: FileOrder,
    worked: Worked,
    rulings: Vec<Ruling<'file>>,
    concluding: Vec<Concluding<'file>>,
}

impl<'file> FileWork<'file> {
    fn worked(&mut self) -> Worked {
        std::mem::take(&mut self.worked)
    }

    fn judged<'run: 'file>(self, settled: Settled<'_, 'run>) -> Worked {
        let order = self.order;
        let watched = self.rulings.into_iter().map(|ruling| (ruling.slot, ruling.pending.judged(settled)));
        let ruled =
            self.concluding.into_iter().map(|concluding| (concluding.slot, concluding.pending.concluded(settled)));
        let judged = watched.chain(ruled);
        judged.map(|(slot, violations)| Worked::filing(slot, order, &violations)).collect()
    }
}

trait Registered {
    const ENTRY: Entry;
}

impl<Lint: FileLint + 'static> Registered for Filed<Lint> {
    const ENTRY: Entry = Entry {
        work: Work::Filed(Filing {
            headline: Lint::HEADLINE,
            reads: Lint::READS,
            source: Source { key: TypeId::of::<Filed<Lint>>, lead: <Filed<Lint> as Leader>::LEAD },
        }),
    };
}

impl<Lint: SweptLint + 'static> Registered for Swept<Lint> {
    const ENTRY: Entry = Entry {
        work: Work::Filed(Filing {
            headline: Lint::HEADLINE,
            reads: Lint::READS,
            source: Source { key: TypeId::of::<Swept<Lint>>, lead: <Swept<Lint> as Leader>::LEAD },
        }),
    };
}

#[derive(Clone, Copy, Debug)]
struct Admitted {
    slot: Slot,
    source: Source,
}

#[derive(Debug)]
pub(crate) struct Filer<'file> {
    claimed: Claimed<'file>,
    rulings: Vec<Ruling<'file>>,
    concluding: Vec<Concluding<'file>>,
    admitted: Vec<Admitted>,
    leads: BTreeMap<TypeId, Lead>,
    starts: Vec<TypeId>,
    finds: Vec<Box<dyn Finds<'file> + 'file>>,
}

impl<'file> Filer<'file> {
    pub(crate) fn started(claimed: Claimed<'file>) -> Filer<'file> {
        let file = <&SourceFile>::from(claimed);
        let admitted: Vec<Admitted> = Lints::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| Some((Slot::from(index), entry.filing()?)))
            .filter_map(|(slot, filing)| {
                Some(Admitted { slot, source: filing.reads.admits(file).map(|_| filing.source)? })
            })
            .collect();
        let leads: BTreeMap<TypeId, Lead> =
            admitted.iter().map(|found| ((found.source.key)(), found.source.lead)).collect();
        let started: Vec<(TypeId, Start)> =
            leads.iter().filter_map(|(key, lead)| Some((*key, lead.swept()?))).collect();
        let finds = started.iter().map(|(_, start)| start(file)).collect();
        let starts = started.into_iter().map(|(key, _)| key).collect();
        let rulings = Lints::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| Some((Slot::from(index), entry.judging()?)))
            .filter_map(|(slot, judging)| Some(Ruling { slot, pending: (judging.pend)(judging.reads.admits(file)?) }))
            .collect();
        let concluding = Lints::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| Some((Slot::from(index), entry.ruling()?)))
            .filter_map(|(slot, rules)| Some(Concluding { slot, pending: (rules.conclude)(rules.reads.admits(file)?) }))
            .collect();
        Filer { claimed, rulings, concluding, admitted, leads, starts, finds }
    }

    fn taken<'found>(sorted: &mut BTreeMap<TypeId, Vec<Violation<'found>>>, source: Source) -> Vec<Violation<'found>> {
        sorted.remove(&(source.key)()).unwrap_or_default()
    }
}

impl<'file> Gathers<FileWork<'file>> for Filer<'file> {
    fn gathered(self, _: &SourceFile) -> FileWork<'file> {
        let Filer { claimed, rulings, concluding, admitted, leads, starts, finds } = self;
        let file = <&SourceFile>::from(claimed);
        let swept: Vec<(TypeId, Found<'_>)> =
            starts.into_iter().zip(finds).map(|(key, watcher)| (key, watcher.found(file))).collect();
        let visited: Worked = swept.iter().map(|(_, sweep)| Worked::visiting(sweep.visited())).collect();
        let listed = swept.into_iter().map(|(key, sweep)| (key, sweep.into_iter().collect()));
        let scanned = leads.iter().filter_map(|(key, lead)| lead.scanned(file).map(|found| (*key, found)));
        let mut sorted: BTreeMap<TypeId, Vec<Violation<'_>>> = listed.chain(scanned).collect();
        let found = admitted.into_iter().map(|found| (found.slot, Self::taken(&mut sorted, found.source)));
        let filed = found.map(|(slot, violations)| Worked::filing(slot, claimed.order(), &violations));
        let worked = filed.chain([visited]).collect();
        FileWork { order: claimed.order(), worked, rulings, concluding }
    }
}

impl<Lint: WatchingLint + 'static> Registered for Judged<Lint>
where
    Judged<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        work: Work::Judged(Judging { headline: Lint::HEADLINE, reads: Lint::READS, pend: Self::pending }),
    };
}

impl<Lint: RuledLint + 'static> Registered for Ruled<Lint>
where
    Ruled<Lint>: Concluded,
{
    const ENTRY: Entry = Entry {
        work: Work::Ruled(Rules { headline: Lint::HEADLINE, reads: Lint::READS, conclude: Self::concluding }),
    };
}

trait Concluded {
    fn concluding(file: &SourceFile) -> Box<dyn Concludes<'_> + '_>;
}

pub(crate) trait Concludes<'file>: fmt::Debug {
    fn concluded<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
struct RuledConclusion<'file, Lint> {
    file: &'file SourceFile,
    lint: PhantomData<Lint>,
}

impl<'file, Lint: RuledLint + fmt::Debug> Concludes<'file> for RuledConclusion<'file, Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn concluded<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        Lint::ruled(self.file, settled.held())
    }
}

impl<Lint: RuledLint + fmt::Debug + 'static> Concluded for Ruled<Lint>
where
    for<'file> RuledConclusion<'file, Lint>: Concludes<'file>,
{
    fn concluding(file: &SourceFile) -> Box<dyn Concludes<'_> + '_> {
        Box::new(RuledConclusion::<Lint> { file, lint: PhantomData })
    }
}

trait Judges {
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_>;
}

pub(crate) trait Awaits<'file>: fmt::Debug {
    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
struct Waiting<'file, Lint: WatchingLint> {
    file: &'file SourceFile,
    watched: Lint::Watched<'file>,
}

impl<'file, Lint: WatchingLint + fmt::Debug> Awaits<'file> for Waiting<'file, Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        let Waiting { file, watched } = *self;
        Lint::judged(file, watched.gathered(file), settled.held())
    }
}

impl<Lint: WatchingLint + fmt::Debug + 'static> Judges for Judged<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_> {
        Box::new(Waiting::<Lint> { file, watched: Lint::watched(file) })
    }
}
