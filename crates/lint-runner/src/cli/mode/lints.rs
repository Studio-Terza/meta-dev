use std::any::TypeId;
use std::collections::BTreeMap;
use std::io::Write;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::thread::Scope;

use cargo_ports::Cargo;

use file_ports::Files;
use lint_core::{Headline, Readout, SourceFile, Violation};
use lint_file::{FileLint, Filed, Judged, Leader, Led, Reads, Ruled, RuledLint, Swept, SweptLint, WatchingLint};
use lint_reader::{Described, Unlocated};
use lint_sweep::{Found, Founds, Gathers, Joined, Joint, Unjoined};
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
use lint_no_dyn::NoDyn;
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

use crate::cli::boundary::{FileOrder, Helper, Helpers, Slot, Worked, Workers};
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

macro_rules! registry {
    () => { Ended };
    ($head:ty $(, $tail:ty)* $(,)?) => { Listed<$head, registry!($($tail),*)> };
}

#[derive(Debug)]
struct Ended;

#[derive(Debug)]
struct Listed<Head, Tail> {
    head: PhantomData<Head>,
    tail: PhantomData<Tail>,
}

type Registry = registry![
    Filed<NoComments>,
    Swept<NoFreeFns>,
    Filed<ThinMain>,
    Swept<VecOutParams>,
    Swept<NamedLifetimes>,
    Judged<EnumPredicates>,
    Swept<ConstShape>,
    Swept<TypeShape>,
    Swept<ExpressionFlow>,
    Swept<FnShape>,
    Swept<LiteralNames>,
    Swept<ManualIteration>,
    Judged<NamedNumbers>,
    Judged<NoForwarders>,
    Swept<NoNumbersInBinaries>,
    Swept<PrivateFns>,
    Filed<ProcessStarters>,
    Ruled<PublicSurface>,
    Judged<PublicApiTests>,
    Judged<NoTestOnlyApi>,
    Swept<NoIgnoredTests>,
    Swept<BoolMethods>,
    Filed<NoDyn>,
];

impl Lints {
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
        let headlines = Registry::headlines(Vec::new());
        let rendered = worked.into_rendered(|slot| slot.picked(&headlines).copied());
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

#[derive(Debug)]
pub(crate) struct FileWork<'file> {
    order: FileOrder,
    worked: Worked,
    file: &'file SourceFile,
}

impl<'file> FileWork<'file> {
    fn worked(&mut self) -> Worked {
        std::mem::take(&mut self.worked)
    }

    fn judged<'run: 'file>(self, settled: Settled<'_, 'run>) -> Worked {
        let judging =
            Judging { file: self.file, order: self.order, slot: Slot::FIRST, settled, worked: Worked::default() };
        Registry::judged(judging).worked
    }
}

trait Registered {
    const HEADLINE: Headline;

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing
    }

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        judging
    }

    fn joined<'file>(
        _: &'file SourceFile,
        tail: impl Joint<Tallied: Founds<'file>> + 'file,
    ) -> impl Joint<Tallied: Founds<'file>> + 'file {
        tail
    }
}

trait Walk {
    fn headlines(headlines: Vec<Headline>) -> Vec<Headline>;

    fn watching(file: &SourceFile) -> impl Joint<Tallied: Founds<'_>> + '_;

    fn filed(filing: Filing<'_>) -> Filing<'_>;

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run>;
}

impl Walk for Ended {
    fn headlines(headlines: Vec<Headline>) -> Vec<Headline> {
        headlines
    }

    fn watching(_: &SourceFile) -> impl Joint<Tallied: Founds<'_>> + '_ {
        Unjoined
    }

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing
    }

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        judging
    }
}

impl<Head: Registered, Tail: Walk> Walk for Listed<Head, Tail> {
    fn headlines(headlines: Vec<Headline>) -> Vec<Headline> {
        Tail::headlines(headlines.into_iter().chain([Head::HEADLINE]).collect())
    }

    fn watching(file: &SourceFile) -> impl Joint<Tallied: Founds<'_>> + '_ {
        Head::joined(file, Tail::watching(file))
    }

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        Tail::filed(Head::filed(filing).advanced())
    }

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        Tail::judged(Head::judged(judging).advanced())
    }
}

#[derive(Debug)]
struct Judging<'file, 'settled, 'run> {
    file: &'file SourceFile,
    order: FileOrder,
    slot: Slot,
    settled: Settled<'settled, 'run>,
    worked: Worked,
}

impl<'file, 'settled, 'run> Judging<'file, 'settled, 'run> {
    fn found(self, violations: Option<Vec<Violation<'file>>>) -> Judging<'file, 'settled, 'run> {
        let found = violations.map(|listed| Worked::filing(self.slot, self.order, &listed));
        Judging { worked: [self.worked].into_iter().chain(found).collect(), ..self }
    }

    fn advanced(self) -> Judging<'file, 'settled, 'run> {
        Judging { slot: self.slot.next(), ..self }
    }
}

#[derive(Clone, Copy, Debug)]
struct Admitted {
    slot: Slot,
    key: TypeId,
}

#[derive(Debug)]
struct Filing<'file> {
    file: &'file SourceFile,
    slot: Slot,
    leds: BTreeMap<TypeId, Led<'file>>,
    admitted: Vec<Admitted>,
    swept: std::vec::IntoIter<Option<Found<'file>>>,
}

impl<'file> Filing<'file> {
    fn led<Leading: Leader>(self, reads: Reads) -> Filing<'file> {
        let Filing { file, slot, leds: earlier, admitted, swept } = self;
        let found = reads.admits(file).map(|_| Admitted { slot, key: TypeId::of::<Leading>() });
        let leds = earlier.into_iter().chain(found.map(|admit| (admit.key, Leading::led(file)))).collect();
        Filing { file, slot, leds, admitted: admitted.into_iter().chain(found).collect(), swept }
    }

    fn swept<Sweeping: 'static>(self) -> Filing<'file> {
        let Filing { file, slot, leds: earlier, admitted, mut swept } = self;
        let found = swept.next().flatten().map(|sweep| (Admitted { slot, key: TypeId::of::<Sweeping>() }, sweep));
        let kept = found.as_ref().map(|(admit, _)| *admit);
        let leds = earlier.into_iter().chain(found.map(|(admit, sweep)| (admit.key, Led::from(sweep)))).collect();
        Filing { file, slot, leds, admitted: admitted.into_iter().chain(kept).collect(), swept }
    }

    fn advanced(self) -> Filing<'file> {
        Filing { slot: self.slot.next(), ..self }
    }

    fn worked(self, order: FileOrder) -> Worked {
        let Filing { mut leds, admitted, .. } = self;
        let visited: Worked = leds.values().filter_map(Led::visited).map(Worked::visiting).collect();
        let filed: Vec<Worked> = admitted
            .into_iter()
            .map(|admit| {
                let violations = leds.get_mut(&admit.key).map_or_default(Led::taken);
                Worked::filing(admit.slot, order, &violations)
            })
            .collect();
        filed.into_iter().chain([visited]).collect()
    }
}

#[derive(Debug)]
pub(crate) struct Filer<'file> {
    claimed: Claimed<'file>,
}

impl<'file> Filer<'file> {
    pub(crate) fn started(claimed: Claimed<'file>) -> Filer<'file> {
        Filer { claimed }
    }
}

impl<'file> Gathers<FileWork<'file>> for Filer<'file> {
    fn gathered(self, _: &SourceFile) -> FileWork<'file> {
        let file = <&SourceFile>::from(self.claimed);
        let order = self.claimed.order();
        let joint = Registry::watching(file);
        let swept = joint.swept(file).founds(Vec::new()).into_iter();
        let filing = Filing { file, slot: Slot::FIRST, leds: BTreeMap::new(), admitted: Vec::new(), swept };
        FileWork { order, worked: Registry::filed(filing).worked(order), file }
    }
}

impl<Lint: FileLint + 'static> Registered for Filed<Lint> {
    const HEADLINE: Headline = Lint::HEADLINE;

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing.led::<Filed<Lint>>(Lint::READS)
    }
}

impl<Lint: SweptLint + 'static> Registered for Swept<Lint> {
    const HEADLINE: Headline = Lint::HEADLINE;

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing.swept::<Swept<Lint>>()
    }

    fn joined<'file>(
        file: &'file SourceFile,
        tail: impl Joint<Tallied: Founds<'file>> + 'file,
    ) -> impl Joint<Tallied: Founds<'file>> + 'file {
        Joined::new(Lint::READS.admits(file).map(Lint::watcher), tail)
    }
}

impl<Lint: WatchingLint> Registered for Judged<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    const HEADLINE: Headline = Lint::HEADLINE;

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        let settled = judging.settled;
        let read = Lint::READS.admits(judging.file);
        judging.found(read.map(|file| Lint::judged(file, Lint::watched(file).gathered(file), settled.held())))
    }
}

impl<Lint: RuledLint> Registered for Ruled<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    const HEADLINE: Headline = Lint::HEADLINE;

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        let settled = judging.settled;
        let read = Lint::READS.admits(judging.file);
        judging.found(read.map(|file| Lint::ruled(file, settled.held())))
    }
}
