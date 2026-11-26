use std::any::TypeId;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::io::Write;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::thread::Scope;

use cargo_ports::Cargo;
use graph::Model;

use lint_hand_impls::HandImpls;

use file_ports::Files;
use lint_core::{FileHead, Headline, Lint, Readout, Report, SourceFile, Violation};
use lint_file::{
    FileLint, Filed, Judged, Kin, KinLint, Leader, Led, Primed, PrimedLint, Reads, Ruled, RuledLint, Swept, SweptLint,
    WatchingLint,
};
use lint_file::{ModelLint, WorkspaceLint};
use lint_port_dependencies::RoleFacts;
use lint_reader::{Described, Unlocated};
use lint_sweep::{Gathers, Member};
use timer_ports::Timer;
use workspace_metadata::Metadata;

use lint_arbitrary_newtypes::ArbitraryNewtypes;
use lint_bool_methods::BoolMethods;
use lint_const_shape::ConstShape;
use lint_crate_graph::CrateGraph;
use lint_distinct_signatures::DistinctSignatures;
use lint_effect_sites::EffectsInPorts;
use lint_enum_predicates::EnumPredicates;
use lint_expression_flow::ExpressionFlow;
use lint_fakes_at_ports::FakesAtPorts;
use lint_fn_shape::FnShape;
use lint_guard_must_use::GuardMustUse;
use lint_laws::HandWrittenLaws;
use lint_linear_flow::{LinearBodies, PlainConditions, PureRouters};
use lint_literal_names::LiteralNames;
use lint_long_exact_strings::LongExactStrings;
use lint_mandatory_laws::MandatoryLaws;
use lint_manual_iteration::ManualIteration;
use lint_module_nesting::ModuleNesting;
use lint_named_invariants::UnnamedPanics;
use lint_named_lifetimes::NamedLifetimes;
use lint_named_numbers::NamedNumbers;
use lint_no_comments::NoComments;
use lint_no_dyn::NoDyn;
use lint_no_early_exits::NoEarlyExits;
use lint_no_forwarders::NoForwarders;
use lint_no_free_fns::NoFreeFns;
use lint_no_ignored_tests::NoIgnoredTests;
use lint_no_numbers_in_binaries::NoNumbersInBinaries;
use lint_no_parameter_bags::NoParameterBags;
use lint_no_plaintext::NoPlaintext;
use lint_no_test_io::NoTestIo;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_node_recursion::NodeRecursion;
use lint_port_batching::PortBatching;
use lint_port_dependencies::{CountedCalls, EffectsInFillers, HiddenSqlx, KeyDependencyRoles, OwnedDependencies};
use lint_primitive_boundary::PrimitiveBoundary;
use lint_private_fns::PrivateFns;
use lint_process_starters::ProcessStarters;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_raw_resources::RawResources;
use lint_repeated_methods::RepeatedMethods;
use lint_repeated_reads::RepeatedReads;
use lint_repeated_shapes::RepeatedShapes;
use lint_rust_only::RustOnly;
use lint_state_graph::{ForeignMatches, StateGraph};
use lint_store_queries::StoreQueries;
use lint_thin_main::ThinMain;
use lint_transition_graph::{AcyclicQuotients, Clustered, Extraction, GraphLint, OneEdge, Transitions, TypeSystem};
use lint_type_shape::TypeShape;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_vec_out_params::VecOutParams;
use lint_workflow_run::WorkflowRun;

use crate::cli::boundary::{FileOrder, Helper, Helpers, Rendered, Slot, Worked, Workers};
use crate::cli::ignored::Ignored;
use crate::cli::sources::{Claimed, Holding, Priming, Settled};
use crate::cli::timings::{Measured, Named, Step, Stopwatch, Worker};
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

type Check = for<'file> fn(&'file Readout) -> Vec<Report<'file>>;

type Query = for<'model> fn(&'model Model) -> Report<'model>;

type Workspaced =
    for<'file, 'head> fn(&'file [FileHead], &'file [Metadata], &'file RoleFacts<'head>, &'file Model) -> Report<'file>;

type Extracting = for<'model> fn(&'model Model) -> Vec<Report<'model>>;

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
    Ruled<ModuleNesting>,
    Judged<NamedNumbers>,
    Judged<NoForwarders>,
    Swept<NoNumbersInBinaries>,
    Modelled<NoParameterBags>,
    Modelled<PrimitiveBoundary>,
    Swept<PrivateFns>,
    Modelled<ProcessStarters>,
    Modelled<RawResources>,
    Modelled<UnnamedPanics>,
    Modelled<HandWrittenLaws>,
    Ruled<PublicSurface>,
    Modelled<RepeatedMethods>,
    Primed<StateGraph>,
    Ruled<GuardMustUse>,
    Judged<PublicApiTests>,
    Grounded<NoTestOnlyApi>,
    Swept<NoIgnoredTests>,
    Modelled<NoTestIo>,
    Modelled<FakesAtPorts>,
    Modelled<PortBatching>,
    OwnedDependencies,
    KeyDependencyRoles,
    Modelled<EffectsInFillers>,
    Kin<CountedCalls>,
    Kin<HiddenSqlx>,
    Modelled<StoreQueries>,
    Swept<LongExactStrings>,
    Ruled<RepeatedShapes>,
    Modelled<MandatoryLaws>,
    Modelled<ForeignMatches>,
    Swept<ArbitraryNewtypes>,
    Swept<BoolMethods>,
    Modelled<EffectsInPorts>,
    Modelled<RepeatedReads>,
    Modelled<NodeRecursion>,
    Swept<NoEarlyExits>,
    Kin<LinearBodies>,
    Kin<PureRouters>,
    Kin<PlainConditions>,
    TypeSystem<'static, 'static>,
    Clustered<AcyclicQuotients>,
    Judged<UnconsumedTraits>,
    RustOnly,
    WorkflowRun,
    Modelled<HandImpls>,
    Modelled<CrateGraph>,
    Modelled<NoPlaintext>,
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
        let entries = Registry::entries(Vec::new());
        let mut rendered = worked.into_rendered(|slot| slot.picked(&entries).and_then(Entry::headline));
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
        let mut jobs: Vec<Job> = Registry::entries(Vec::new())
            .into_iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.queued(Slot::from(index)))
            .collect();
        jobs.sort_by_key(Job::rank);
        let claimed = sources.queue().claimed(&jobs).map(|job| job.done(readout, sources));
        let queued: Worked = claimed.collect();
        let waiting = Stopwatch::started(sources.wired().timer());
        let settled = Settled::from(sources.gathering());
        let waited = waiting.stopped(Step::Judged(worker).waited());
        let judging = Stopwatch::started(sources.wired().timer());
        let judged: Worked = work.into_iter().map(|file| file.judged(settled)).collect();
        let judged_timing = judging.stopped(Step::Judged(worker));
        let timings = lowered.timings().iter().copied().chain([waited, judged_timing]);
        timings.map(Worked::timed).chain([filed, queued, judged]).collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    name: Named,
    work: Work,
}

#[derive(Clone, Copy, Debug)]
enum Work {
    Checked(Check),
    Modelled(Query),
    Extracted(Extracting),
    Filed(Headline),
    Described(Workspaced),
    Composite(Headline, Workspaced),
}

impl Entry {
    pub(crate) const fn spelled<Lint: ?Sized>(work: Work) -> Entry {
        Entry { name: Named::spelled::<Lint>(), work }
    }

    fn queued(self, slot: Slot) -> Option<Job> {
        match self.work {
            Work::Filed(_) => None,
            Work::Checked(_) | Work::Modelled(_) | Work::Extracted(_) | Work::Described(_) | Work::Composite(..) => {
                Some(Job { slot, step: Step::Lint(self.name), work: self.work })
            }
        }
    }

    pub(crate) fn headline(&self) -> Option<Headline> {
        match self.work {
            Work::Filed(headline) | Work::Composite(headline, _) => Some(headline),
            Work::Checked(_) | Work::Modelled(_) | Work::Extracted(_) | Work::Described(_) => None,
        }
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

impl Work {
    fn awaited<'run>(self, sources: &'run Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Ready<'run> {
        match self {
            Work::Modelled(_) | Work::Extracted(_) => Ready::Modelled(sources.gathering().model()),
            Work::Described(_) | Work::Composite(..) => Ready::Grounded(Ground::awaited(sources)),
            Work::Checked(_) | Work::Filed(_) => Ready::Bare,
        }
    }

    fn reported<'run>(self, readout: &'run Readout, ready: Ready<'run>) -> Vec<Report<'run>> {
        match self {
            Work::Checked(check) => check(readout),
            Work::Modelled(query) => ready.model().map(query).into_iter().collect(),
            Work::Described(_) | Work::Composite(..) | Work::Extracted(_) | Work::Filed(_) => Vec::new(),
        }
    }

    fn worked<'run>(self, slot: Slot, readout: &'run Readout, ready: Ready<'run>) -> Worked {
        match self {
            Work::Extracted(_) | Work::Described(_) | Work::Composite(..) => self.grounded(slot, ready),
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) => {
                Worked::rendering(slot, self.reported(readout, ready))
            }
        }
    }

    fn grounded(self, slot: Slot, ready: Ready<'_>) -> Worked {
        match self {
            Work::Extracted(extracted) => Worked::rendering(slot, ready.model().map_or_default(extracted)),
            Work::Described(described) => {
                Worked::rendering(slot, ready.ground().map(|ground| ground.reported(described)).into_iter().collect())
            }
            Work::Composite(_, described) => ready.filed(slot, described),
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) => Worked::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Early(Reverse<Slot>),
    Whole(Slot),
    Late(Slot),
}

#[derive(Clone, Copy, Debug)]
struct Job {
    slot: Slot,
    step: Step,
    work: Work,
}

impl Job {
    fn rank(&self) -> Rank {
        match self.work {
            Work::Checked(_) | Work::Filed(_) => Rank::Early(Reverse(self.slot)),
            Work::Extracted(_) => Rank::Whole(self.slot),
            Work::Modelled(_) | Work::Described(_) | Work::Composite(..) => Rank::Late(self.slot),
        }
    }

    fn done(self, readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Worked {
        let waiting = Stopwatch::started(sources.wired().timer());
        let ready = self.work.awaited(sources);
        let waited = waiting.stopped(self.step.waited());
        let stopwatch = Stopwatch::started(sources.wired().timer());
        let worked = self.work.worked(self.slot, readout, ready);
        [worked, Worked::timed(waited), Worked::timed(stopwatch.stopped(self.step))].into_iter().collect()
    }
}

trait Registered {
    const ENTRY: Entry;

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing
    }

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        judging
    }
}

trait Walk {
    fn entries(entries: Vec<Entry>) -> Vec<Entry>;

    fn filed(filing: Filing<'_>) -> Filing<'_>;

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run>;
}

impl Walk for Ended {
    fn entries(entries: Vec<Entry>) -> Vec<Entry> {
        entries
    }

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing
    }

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        judging
    }
}

impl<Head: Registered, Tail: Walk> Walk for Listed<Head, Tail> {
    fn entries(entries: Vec<Entry>) -> Vec<Entry> {
        Tail::entries(entries.into_iter().chain([Head::ENTRY]).collect())
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
    member: Member,
}

#[derive(Debug)]
struct Filing<'file> {
    file: &'file SourceFile,
    slot: Slot,
    leds: BTreeMap<TypeId, Led<'file>>,
    admitted: Vec<Admitted>,
}

impl<'file> Filing<'file> {
    fn led<Leading: Leader>(self, reads: Reads, member: Member) -> Filing<'file> {
        let Filing { file, slot, leds: started, mut admitted } = self;
        let found = reads.admits(file).map(|_| Admitted { slot, key: TypeId::of::<Leading>(), member });
        let leds = found.iter().fold(started, |mut kept, admit| {
            kept.entry(admit.key).or_insert_with(|| Leading::led(file));
            kept
        });
        admitted.extend(found);
        Filing { file, slot, leds, admitted }
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
                let violations = leds.get_mut(&admit.key).map_or_default(|led| led.member(admit.member));
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
        let filing = Filing { file, slot: Slot::FIRST, leds: BTreeMap::new(), admitted: Vec::new() };
        FileWork { order, worked: Registry::filed(filing).worked(order), file }
    }
}

impl<Lint: FileLint + 'static> Registered for Filed<Lint> {
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing.led::<Filed<Lint>>(Lint::READS, Member::FIRST)
    }
}

impl<Lint: SweptLint + 'static> Registered for Swept<Lint> {
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing.led::<Swept<Lint>>(Lint::READS, Member::FIRST)
    }
}

impl<Lint: KinLint + 'static> Registered for Kin<Lint> {
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

    fn filed(filing: Filing<'_>) -> Filing<'_> {
        filing.led::<Lint::Leader>(Lint::READS, Lint::MEMBER)
    }
}

impl<Lint: WatchingLint> Registered for Judged<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

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
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        let settled = judging.settled;
        let read = Lint::READS.admits(judging.file);
        judging.found(read.map(|file| Lint::ruled(file, settled.held())))
    }
}

impl<Lint: PrimedLint> Registered for Primed<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Priming<'run, Lint>,
{
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Filed(Lint::HEADLINE));

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        let settled = judging.settled;
        let read = Lint::READS.admits(judging.file);
        judging.found(read.map(|file| Lint::judged(file, settled.primer(), settled.primed())))
    }
}

#[derive(Debug)]
struct Modelled<Lint>(PhantomData<Lint>);

impl<Lint: ModelLint> Registered for Modelled<Lint> {
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Modelled(<Lint as ModelLint>::report));
}

impl Registered for OwnedDependencies {
    const ENTRY: Entry = Entry::spelled::<OwnedDependencies>(Work::Described(|heads, workspaces, _, model| {
        <OwnedDependencies as WorkspaceLint>::report(heads, workspaces, &(), model)
    }));
}

impl Registered for KeyDependencyRoles {
    const ENTRY: Entry = Entry::spelled::<KeyDependencyRoles>(Work::Described(|heads, workspaces, facts, model| {
        <KeyDependencyRoles as WorkspaceLint>::report(heads, workspaces, facts, model)
    }));
}

impl Registered for TypeSystem<'static, 'static> {
    const ENTRY: Entry = Entry::spelled::<TypeSystem>(Work::Extracted(|model| {
        let extraction = Extraction::default();
        let transitions = Transitions::from(model);
        let system = TypeSystem::built_from(&extraction, &transitions);
        [OneEdge::dirty(&system), DistinctSignatures::dirty(&system)].into_iter().flatten().collect()
    }));
}

impl Registered for Clustered<AcyclicQuotients> {
    const ENTRY: Entry = Entry::spelled::<Clustered<AcyclicQuotients>>(Work::Extracted(|model| {
        Some(<Clustered<AcyclicQuotients> as ModelLint>::report(model))
            .filter(|report| report.dirty().is_some())
            .into_iter()
            .collect()
    }));
}

impl Registered for RustOnly {
    const ENTRY: Entry =
        Entry::spelled::<RustOnly>(Work::Checked(|readout| vec![RustOnly::report(readout.repo_files())]));
}

impl Registered for WorkflowRun {
    const ENTRY: Entry =
        Entry::spelled::<WorkflowRun>(Work::Checked(|readout| vec![WorkflowRun::report(readout.repo_files())]));
}

#[derive(Clone, Copy, Debug)]
struct Ground<'run> {
    heads: &'run [FileHead],
    described: &'run Described,
    roles: &'run RoleFacts<'run>,
    model: &'run Model,
}

impl<'run> Ground<'run> {
    fn awaited(sources: &'run Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Option<Ground<'run>> {
        let described = sources.located().done().as_ref().ignoring(Ignored::Unlocated)?;
        let roles = Settled::from(sources.gathering()).roles();
        let model = sources.gathering().model()?;
        Some(Ground { heads: sources.gathering().heads(), described, roles, model })
    }

    fn reported(self, described: Workspaced) -> Report<'run> {
        described(self.heads, Described::workspaces(self.described), self.roles, self.model)
    }
}

#[derive(Clone, Copy, Debug)]
enum Ready<'run> {
    Bare,
    Modelled(Option<&'run Model>),
    Grounded(Option<Ground<'run>>),
}

impl<'run> Ready<'run> {
    fn model(self) -> Option<&'run Model> {
        match self {
            Ready::Modelled(model) => model,
            Ready::Grounded(ground) => ground.map(|found| found.model),
            Ready::Bare => None,
        }
    }

    fn filed(self, slot: Slot, described: Workspaced) -> Worked {
        let report = self.ground().map(|ground| ground.reported(described));
        report.map_or_default(|found| Worked::filing(slot, FileOrder::LAST, found.violations()))
    }

    fn ground(self) -> Option<Ground<'run>> {
        match self {
            Ready::Grounded(ground) => ground,
            Ready::Modelled(_) | Ready::Bare => None,
        }
    }
}

#[derive(Debug)]
struct Grounded<Lint>(PhantomData<Lint>);

impl<Lint> Registered for Grounded<Lint>
where
    Lint: WatchingLint + for<'head> WorkspaceLint<Facts<'head> = ()> + 'static,
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    const ENTRY: Entry = Entry::spelled::<Lint>(Work::Composite(Lint::HEADLINE, |heads, workspaces, _, model| {
        Lint::report(heads, workspaces, &(), model)
    }));

    fn judged<'file, 'settled, 'run: 'file>(judging: Judging<'file, 'settled, 'run>) -> Judging<'file, 'settled, 'run> {
        <Judged<Lint> as Registered>::judged(judging)
    }
}
