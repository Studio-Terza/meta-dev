use std::any::TypeId;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::thread::Scope;

use cargo_ports::Cargo;
use graph::Model;

use lint_hand_impls::HandImpls;

use file_ports::Files;
use lint_core::{FileHead, Headline, Lint, ModelLint, Readout, Report, SourceFile, Violation, WorkspaceLint};
use lint_file::{
    ContextLint, FileLint, Filed, Judged, Kin, KinLint, Lead, Leader, OneWalk, Primed, PrimedLint, Reads, SortStart,
    Start, Swept, SweptLint,
};
use lint_port_dependencies::{RoleFacts, RoleReading};
use lint_reader::{Described, Unlocated};
use lint_sweep::{Finds, Gathers, Member, Sorts, Watching};
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
use lint_linear_flow::{LinearBodies, PlainConditions, PureRouters};
use lint_literal_names::LiteralNames;
use lint_long_exact_strings::LongExactStrings;
use lint_mandatory_laws::MandatoryLaws;
use lint_manual_iteration::ManualIteration;
use lint_module_nesting::ModuleNesting;
use lint_named_lifetimes::NamedLifetimes;
use lint_named_numbers::NamedNumbers;
use lint_no_comments::NoComments;
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
use lint_state_graph::StateGraph;
use lint_store_queries::StoreQueries;
use lint_thin_main::ThinMain;
use lint_transition_graph::{AcyclicQuotients, Clustered, Extraction, GraphLint, OneEdge, Transitions, TypeSystem};
use lint_type_shape::TypeShape;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_vec_out_params::VecOutParams;
use lint_workflow_run::WorkflowRun;

use super::Survey;
use crate::cli::boundary::{FileOrder, Helper, Helpers, Rendered, Slot, Surveyed, Worked, Workers};
use crate::cli::sources::{Claimed, Gleans, Holding, Priming, Settled};
use crate::cli::timings::{Measured, Named, Step, Stopwatch, Worker};
use crate::cli::{Outcome, Sources, Verdict};

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
        sources: &Sources<impl Files, impl Cargo, impl Timer>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        let mine = Lints::worked(readout.done(), sources, Worker::MAIN);
        let worked: Worked = [mine].into_iter().chain(self.helpers.into_iter().map(Helper::joined)).collect();
        let first = sources.earlier().chain([readout.timing()]);
        let table = sources.wired().table(first.chain(worked.timings().iter().copied()));
        let surveyed = Surveyed::from(&worked);
        let outcome = Lints::gathered(readout.done(), sources.located().done().as_ref().err(), worked);
        outcome.surveyed(&[format_args!("{surveyed}{table}")], out, err)
    }
}

type Check = for<'file> fn(&'file Readout) -> Vec<Report<'file>>;

type Pend = for<'file> fn(&'file SourceFile) -> Box<dyn Awaits<'file> + 'file>;

#[derive(Clone, Copy, Debug)]
struct Judging {
    headline: Headline,
    reads: Reads,
    pend: Pend,
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
    member: Member,
}

type Query = for<'model> fn(&'model Model) -> Report<'model>;

type Workspaced =
    for<'file, 'head> fn(&'file [FileHead], &'file [Metadata], &'file RoleFacts<'head>, &'file Model) -> Report<'file>;

type Extracting = for<'model> fn(&'model Model) -> Vec<Report<'model>>;

impl Lints {
    const LINTS: [Entry; 57] = [
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
        <Judged<ModuleNesting> as Registered>::ENTRY,
        <Judged<NamedNumbers> as Registered>::ENTRY,
        <Judged<NoForwarders> as Registered>::ENTRY,
        <Swept<NoNumbersInBinaries> as Registered>::ENTRY,
        <Judged<NoParameterBags> as Registered>::ENTRY,
        Entry::spelled::<PrimitiveBoundary>(Work::Modelled(<PrimitiveBoundary as ModelLint>::report)),
        <Swept<PrivateFns> as Registered>::ENTRY,
        Entry::spelled::<ProcessStarters>(Work::Modelled(<ProcessStarters as ModelLint>::report)),
        Entry::spelled::<RawResources>(Work::Modelled(<RawResources as ModelLint>::report)),
        <Judged<PublicSurface> as Registered>::ENTRY,
        <Judged<RepeatedMethods> as Registered>::ENTRY,
        <Primed<StateGraph> as Registered>::ENTRY,
        <Judged<GuardMustUse> as Registered>::ENTRY,
        <Judged<PublicApiTests> as Registered>::ENTRY,
        <Judged<NoTestOnlyApi> as Registered>::ENTRY,
        <Swept<NoIgnoredTests> as Registered>::ENTRY,
        Entry::spelled::<NoTestIo>(Work::Modelled(<NoTestIo as ModelLint>::report)),
        Entry::spelled::<FakesAtPorts>(Work::Modelled(<FakesAtPorts as ModelLint>::report)),
        Entry::spelled::<PortBatching>(Work::Modelled(<PortBatching as ModelLint>::report)),
        Entry::spelled::<OwnedDependencies>(Work::Described(|heads, workspaces, _, model| {
            <OwnedDependencies as WorkspaceLint>::report(heads, workspaces, &(), model)
        })),
        Entry::spelled::<KeyDependencyRoles>(Work::Described(|heads, workspaces, facts, model| {
            <KeyDependencyRoles as WorkspaceLint>::report(heads, workspaces, facts, model)
        })),
        Entry::spelled::<EffectsInFillers>(Work::Modelled(<EffectsInFillers as ModelLint>::report)),
        <Kin<CountedCalls> as Registered>::ENTRY,
        <Kin<HiddenSqlx> as Registered>::ENTRY,
        Entry::spelled::<StoreQueries>(Work::Modelled(<StoreQueries as ModelLint>::report)),
        <Swept<LongExactStrings> as Registered>::ENTRY,
        <Judged<RepeatedShapes> as Registered>::ENTRY,
        Entry::spelled::<MandatoryLaws>(Work::Modelled(<MandatoryLaws as ModelLint>::report)),
        <Swept<ArbitraryNewtypes> as Registered>::ENTRY,
        <Swept<BoolMethods> as Registered>::ENTRY,
        Entry::spelled::<EffectsInPorts>(Work::Modelled(<EffectsInPorts as ModelLint>::report)),
        Entry::spelled::<RepeatedReads>(Work::Modelled(<RepeatedReads as ModelLint>::report)),
        Entry::spelled::<NodeRecursion>(Work::Modelled(<NodeRecursion as ModelLint>::report)),
        <Judged<OneWalk> as Registered>::ENTRY,
        <Swept<NoEarlyExits> as Registered>::ENTRY,
        <Kin<LinearBodies> as Registered>::ENTRY,
        <Kin<PureRouters> as Registered>::ENTRY,
        <Kin<PlainConditions> as Registered>::ENTRY,
        Entry::spelled::<TypeSystem>(Work::Extracted(|model| {
            let extraction = Extraction::default();
            let transitions = Transitions::from(model);
            let system = TypeSystem::built_from(&extraction, &transitions);
            [OneEdge::dirty(&system), <Clustered<AcyclicQuotients>>::dirty(&system), DistinctSignatures::dirty(&system)]
                .into_iter()
                .flatten()
                .collect()
        })),
        <Judged<UnconsumedTraits> as Registered>::ENTRY,
        Entry::spelled::<RustOnly>(Work::Checked(|readout| vec![RustOnly::report(readout.repo_files())])),
        Entry::spelled::<WorkflowRun>(Work::Checked(|readout| vec![WorkflowRun::report(readout.repo_files())])),
        Entry::spelled::<HandImpls>(Work::Modelled(<HandImpls as ModelLint>::report)),
        Entry::spelled::<CrateGraph>(Work::Modelled(<CrateGraph as ModelLint>::report)),
        Entry::spelled::<NoPlaintext>(Work::Modelled(<NoPlaintext as ModelLint>::report)),
    ];

    pub(crate) fn started<'scope, F: Files + Sync, C: Cargo + Sync, T: Timer + Sync>(
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, C, T>,
    ) -> Crew<'scope> {
        let helpers = Workers::available()
            .helpers()
            .filter_map(|index| Helper::spawned(scope, move || Self::helped(sources, Worker::from(index))).ok());
        let locating = Helper::spawned(scope, move || Self::locating(sources)).ok();
        Crew { helpers: locating.into_iter().chain(helpers).collect() }
    }

    fn gathered<'run>(readout: &'run Readout, unlocated: Option<&'run Unlocated>, worked: Worked) -> Outcome<'run> {
        let mut rendered = worked.into_rendered(|slot| slot.picked(&Self::LINTS).and_then(Entry::headline));
        rendered.sort_by_key(Rendered::slot);
        unlocated.map_or_else(|| Outcome::from_rendered(rendered, readout.failures()), Outcome::Unlocated)
    }

    fn locating(sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        Worked::timed(sources.located().timing())
    }

    fn helped(sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let readout = ManuallyDrop::new(sources.read(worker));
        [Self::worked(readout.done(), sources, worker), Worked::timed(readout.timing())].into_iter().collect()
    }

    fn worked(readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let mut lowered = sources.gathered(readout, worker, Filer::started);
        let mut work: Vec<FileWork<'_>> = lowered.filed();
        let filed: Worked = work.iter_mut().map(FileWork::worked).collect();
        let mut jobs: Vec<Job> = Self::LINTS
            .into_iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.queued(Slot::from(index)))
            .chain([Job::SURVEY])
            .collect();
        jobs.sort_by_key(Job::rank);
        let claimed = sources.queue().claimed(&jobs).map(|job| job.done(readout, sources));
        let queued: Worked = claimed.collect();
        let judging = Stopwatch::started(sources.wired().timer());
        let settled = Settled::from(sources.gathering());
        let judged: Worked = work.into_iter().map(|file| file.judged(settled)).collect();
        let judged_timing = judging.stopped(Step::Judged(worker));
        let timings = lowered.timings().iter().copied().chain([judged_timing]);
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
    Filed(Filing),
    Judged(Judging),
    Described(Workspaced),
    Survey,
}

impl Entry {
    pub(crate) const fn spelled<Lint: ?Sized>(work: Work) -> Entry {
        Entry { name: Named::spelled::<Lint>(), work }
    }

    fn queued(self, slot: Slot) -> Option<Job> {
        match self.work {
            Work::Filed(_) | Work::Judged(_) => None,
            Work::Checked(_) | Work::Modelled(_) | Work::Extracted(_) | Work::Described(_) | Work::Survey => {
                Some(Job { slot, step: Step::Lint(self.name), work: self.work })
            }
        }
    }

    fn filing(self) -> Option<Filing> {
        match self.work {
            Work::Filed(filing) => Some(filing),
            Work::Checked(_)
            | Work::Modelled(_)
            | Work::Extracted(_)
            | Work::Described(_)
            | Work::Judged(_)
            | Work::Survey => None,
        }
    }

    fn judging(self) -> Option<Judging> {
        match self.work {
            Work::Judged(judging) => Some(judging),
            Work::Checked(_)
            | Work::Modelled(_)
            | Work::Extracted(_)
            | Work::Described(_)
            | Work::Filed(_)
            | Work::Survey => None,
        }
    }

    fn headline(&self) -> Option<Headline> {
        self.filing().map(|filing| filing.headline).or_else(|| self.judging().map(|judging| judging.headline))
    }
}

#[derive(Debug)]
struct Ruling<'file> {
    slot: Slot,
    pending: Box<dyn Awaits<'file> + 'file>,
}

#[derive(Debug)]
pub(crate) struct FileWork<'file> {
    order: FileOrder,
    worked: Worked,
    rulings: Vec<Ruling<'file>>,
}

impl<'file> FileWork<'file> {
    fn worked(&mut self) -> Worked {
        std::mem::take(&mut self.worked)
    }

    fn judged<'run: 'file>(self, settled: Settled<'_, 'run>) -> Worked {
        let order = self.order;
        let judged = self.rulings.into_iter().map(|ruling| (ruling.slot, ruling.pending.judged(settled)));
        judged.map(|(slot, violations)| Worked::filing(slot, order, &violations)).collect()
    }
}

impl Work {
    fn reported<'run>(
        self,
        readout: &'run Readout,
        sources: &'run Sources<impl Files, impl Cargo, impl Timer>,
    ) -> Vec<Report<'run>> {
        match self {
            Work::Checked(check) => check(readout),
            Work::Modelled(query) => sources.gathering().model().map(query).into_iter().collect(),
            Work::Described(_) | Work::Extracted(_) | Work::Survey | Work::Filed(_) | Work::Judged(_) => Vec::new(),
        }
    }

    fn worked(self, slot: Slot, readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        match self {
            Work::Survey | Work::Extracted(_) | Work::Described(_) => self.settled(slot, sources),
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) | Work::Judged(_) => {
                Worked::rendering(slot, self.reported(readout, sources))
            }
        }
    }

    fn settled(self, slot: Slot, sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        match self {
            Work::Survey => {
                Worked::surveying(&Survey::settled(&Settled::from(sources.gathering()), sources.gathering().model()))
            }
            Work::Extracted(_) | Work::Described(_) => self.grounded(slot, sources, Settled::from(sources.gathering())),
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) | Work::Judged(_) => Worked::default(),
        }
    }

    fn grounded(
        self,
        slot: Slot,
        sources: &Sources<impl Files, impl Cargo, impl Timer>,
        settled: Settled<'_, '_>,
    ) -> Worked {
        match self {
            Work::Extracted(extracted) => {
                Worked::rendering(slot, sources.gathering().model().map_or_default(extracted))
            }
            Work::Described(described) => Worked::rendering(
                slot,
                sources
                    .located()
                    .done()
                    .as_ref()
                    .ok()
                    .zip(sources.gathering().model())
                    .map(|(located, model)| {
                        let facts = Gleans::<RoleReading>::gleaned(&settled);
                        described(sources.gathering().heads(), Described::workspaces(located), facts, model)
                    })
                    .into_iter()
                    .collect(),
            ),
            Work::Checked(_) | Work::Modelled(_) | Work::Survey | Work::Filed(_) | Work::Judged(_) => Worked::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Survey,
    Early(Reverse<Slot>),
    Late(Slot),
}

#[derive(Clone, Copy, Debug)]
struct Job {
    slot: Slot,
    step: Step,
    work: Work,
}

impl Job {
    const SURVEY: Job = Job { slot: Slot::NONE, step: Step::Survey, work: Work::Survey };

    fn rank(&self) -> Rank {
        match self.work {
            Work::Survey => Rank::Survey,
            Work::Checked(_) | Work::Filed(_) | Work::Judged(_) => Rank::Early(Reverse(self.slot)),
            Work::Modelled(_) | Work::Extracted(_) | Work::Described(_) => Rank::Late(self.slot),
        }
    }

    fn done(self, readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        let stopwatch = Stopwatch::started(sources.wired().timer());
        let worked = self.work.worked(self.slot, readout, sources);
        [worked, Worked::timed(stopwatch.stopped(self.step))].into_iter().collect()
    }
}

trait Registered {
    const ENTRY: Entry;
}

impl<Lint: FileLint + 'static> Registered for Filed<Lint> {
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Filed(Filing {
            headline: Lint::HEADLINE,
            reads: Lint::READS,
            source: Source {
                key: TypeId::of::<Filed<Lint>>,
                lead: <Filed<Lint> as Leader>::LEAD,
                member: Member::FIRST,
            },
        }),
    };
}

impl<Lint: SweptLint + 'static> Registered for Swept<Lint> {
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Filed(Filing {
            headline: Lint::HEADLINE,
            reads: Lint::READS,
            source: Source {
                key: TypeId::of::<Swept<Lint>>,
                lead: <Swept<Lint> as Leader>::LEAD,
                member: Member::FIRST,
            },
        }),
    };
}

impl<Lint: KinLint + 'static> Registered for Kin<Lint> {
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Filed(Filing {
            headline: Lint::HEADLINE,
            reads: Lint::READS,
            source: Source {
                key: TypeId::of::<Lint::Leader>,
                lead: <Lint::Leader as Leader>::LEAD,
                member: Lint::MEMBER,
            },
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
    admitted: Vec<Admitted>,
    leads: BTreeMap<TypeId, Lead>,
    starts: Vec<TypeId>,
    sorting: Vec<TypeId>,
    finds: Vec<Box<dyn Finds<'file> + 'file>>,
    sorts: Vec<Box<dyn Sorts<'file> + 'file>>,
}

impl<'file> Filer<'file> {
    pub(crate) fn started(claimed: Claimed<'file>) -> Filer<'file> {
        let file = claimed.file();
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
        let sorted: Vec<(TypeId, SortStart)> =
            leads.iter().filter_map(|(key, lead)| Some((*key, lead.sorting()?))).collect();
        let finds = started.iter().map(|(_, start)| start(file)).collect();
        let sorts = sorted.iter().map(|(_, start)| start(file)).collect();
        let starts = started.into_iter().map(|(key, _)| key).collect();
        let sorting = sorted.into_iter().map(|(key, _)| key).collect();
        let rulings = Lints::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| Some((Slot::from(index), entry.judging()?)))
            .filter_map(|(slot, judging)| Some(Ruling { slot, pending: (judging.pend)(judging.reads.admits(file)?) }))
            .collect();
        Filer { claimed, rulings, admitted, leads, starts, sorting, finds, sorts }
    }

    fn taken<'found>(
        sorted: &mut BTreeMap<TypeId, Vec<Vec<Violation<'found>>>>,
        source: Source,
    ) -> Vec<Violation<'found>> {
        let kept = sorted.get_mut(&(source.key)()).and_then(|lists| lists.get_mut(source.member.index()));
        kept.map_or_default(std::mem::take)
    }
}

impl<'file> Gathers<FileWork<'file>> for Filer<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        let found = self.finds.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() });
        let sorted = found.chain(self.sorts.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }));
        sorted.chain(self.rulings.iter_mut().flat_map(|ruling| ruling.pending.watching())).collect()
    }

    fn gathered(self) -> FileWork<'file> {
        let Filer { claimed, rulings, admitted, leads, starts, sorting, finds, sorts } = self;
        let file = claimed.file();
        let listed = starts.into_iter().zip(finds).map(|(key, watcher)| (key, vec![watcher.found()]));
        let watched = listed.chain(sorting.into_iter().zip(sorts).map(|(key, watcher)| (key, watcher.sorted())));
        let scanned = leads.iter().filter_map(|(key, lead)| lead.scanned(file).map(|found| (*key, found)));
        let mut sorted: BTreeMap<TypeId, Vec<Vec<Violation<'_>>>> = watched.chain(scanned).collect();
        let found = admitted.into_iter().map(|found| (found.slot, Self::taken(&mut sorted, found.source)));
        let worked = found.map(|(slot, violations)| Worked::filing(slot, claimed.order(), &violations)).collect();
        FileWork { order: claimed.order(), worked, rulings }
    }
}

impl<Lint: ContextLint + 'static> Registered for Judged<Lint>
where
    Judged<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Judged(Judging { headline: Lint::HEADLINE, reads: Lint::READS, pend: Self::pending }),
    };
}

impl<Lint: PrimedLint + 'static> Registered for Primed<Lint>
where
    Primed<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Judged(Judging { headline: Lint::HEADLINE, reads: Lint::READS, pend: Self::pending }),
    };
}

trait Judges {
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_>;
}

pub(crate) trait Awaits<'file>: fmt::Debug {
    fn watching(&mut self) -> Vec<&mut Watching<'_>>;

    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
struct Waiting<'file, Lint: ContextLint> {
    file: &'file SourceFile,
    watched: Lint::Watched<'file>,
}

#[derive(Debug)]
struct PrimedWaiting<'file, Lint: PrimedLint> {
    file: &'file SourceFile,
    watched: Lint::Watched<'file>,
}

impl<'file, Lint: ContextLint + fmt::Debug> Awaits<'file> for Waiting<'file, Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watched.watching()
    }

    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        let Waiting { file, watched } = *self;
        Lint::judged(file, watched.gathered(), settled.held())
    }
}

impl<'file, Lint: PrimedLint + fmt::Debug> Awaits<'file> for PrimedWaiting<'file, Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Priming<'run, Lint>,
{
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watched.watching()
    }

    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        let PrimedWaiting { file, watched } = *self;
        Lint::judged(file, watched.gathered(), settled.primer(), settled.primed())
    }
}

impl<Lint: ContextLint + fmt::Debug + 'static> Judges for Judged<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_> {
        Box::new(Waiting::<Lint> { file, watched: Lint::watched(file) })
    }
}

impl<Lint: PrimedLint + fmt::Debug + 'static> Judges for Primed<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Priming<'run, Lint>,
{
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_> {
        Box::new(PrimedWaiting::<Lint> { file, watched: Lint::watched(file) })
    }
}
