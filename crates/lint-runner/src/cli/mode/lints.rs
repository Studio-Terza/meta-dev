use lint_core::Sourced;
use std::any::TypeId;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt;
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
    FileLint, Filed, Judged, Kin, KinLint, Lead, Leader, Primed, PrimedLint, Reads, Ruled, RuledLint, SortStart, Start,
    Swept, SweptLint, WatchingLint,
};
use lint_file::{ModelLint, WorkspaceLint};
use lint_port_dependencies::RoleFacts;
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
use lint_state_graph::{ForeignMatches, StateGraph};
use lint_store_queries::StoreQueries;
use lint_thin_main::ThinMain;
use lint_transition_graph::{AcyclicQuotients, Clustered, Extraction, GraphLint, OneEdge, Transitions, TypeSystem};
use lint_type_shape::TypeShape;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_vec_out_params::VecOutParams;
use lint_workflow_run::WorkflowRun;

use crate::cli::boundary::{FileOrder, Helper, Helpers, Rendered, Slot, Worked, Workers};
use crate::cli::sources::{Claimed, Holding, Priming, Settled};
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
        sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        let mine = Lints::worked(readout.done(), sources, Worker::MAIN);
        let worked: Worked = [mine].into_iter().chain(self.helpers.into_iter().map(Helper::joined)).collect();
        let first = sources.earlier().chain([readout.timing()]);
        let table = sources.wired().table(first.chain(worked.timings().iter().copied()));
        let outcome = Lints::gathered(readout.done(), sources.located().done().as_ref().err(), worked);
        outcome.surveyed(&[table], out, err)
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
    member: Member,
}

type Query = for<'model> fn(&'model Model) -> Report<'model>;

type Workspaced =
    for<'file, 'head> fn(&'file [FileHead], &'file [Metadata], &'file RoleFacts<'head>, &'file Model) -> Report<'file>;

type Extracting = for<'model> fn(&'model Model) -> Vec<Report<'model>>;

impl Lints {
    const LINTS: [Entry; 58] = [
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
        <Ruled<ModuleNesting> as Registered>::ENTRY,
        <Judged<NamedNumbers> as Registered>::ENTRY,
        <Judged<NoForwarders> as Registered>::ENTRY,
        <Swept<NoNumbersInBinaries> as Registered>::ENTRY,
        Entry::spelled::<NoParameterBags>(Work::Modelled(<NoParameterBags as ModelLint>::report)),
        Entry::spelled::<PrimitiveBoundary>(Work::Modelled(<PrimitiveBoundary as ModelLint>::report)),
        <Swept<PrivateFns> as Registered>::ENTRY,
        Entry::spelled::<ProcessStarters>(Work::Modelled(<ProcessStarters as ModelLint>::report)),
        Entry::spelled::<RawResources>(Work::Modelled(<RawResources as ModelLint>::report)),
        <Ruled<PublicSurface> as Registered>::ENTRY,
        Entry::spelled::<RepeatedMethods>(Work::Modelled(<RepeatedMethods as ModelLint>::report)),
        <Primed<StateGraph> as Registered>::ENTRY,
        <Ruled<GuardMustUse> as Registered>::ENTRY,
        <Judged<PublicApiTests> as Registered>::ENTRY,
        <Grounded<NoTestOnlyApi> as Registered>::ENTRY,
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
        <Ruled<RepeatedShapes> as Registered>::ENTRY,
        Entry::spelled::<MandatoryLaws>(Work::Modelled(<MandatoryLaws as ModelLint>::report)),
        Entry::spelled::<ForeignMatches>(Work::Modelled(<ForeignMatches as ModelLint>::report)),
        <Swept<ArbitraryNewtypes> as Registered>::ENTRY,
        <Swept<BoolMethods> as Registered>::ENTRY,
        Entry::spelled::<EffectsInPorts>(Work::Modelled(<EffectsInPorts as ModelLint>::report)),
        Entry::spelled::<RepeatedReads>(Work::Modelled(<RepeatedReads as ModelLint>::report)),
        Entry::spelled::<NodeRecursion>(Work::Modelled(<NodeRecursion as ModelLint>::report)),
        <Swept<NoEarlyExits> as Registered>::ENTRY,
        <Kin<LinearBodies> as Registered>::ENTRY,
        <Kin<PureRouters> as Registered>::ENTRY,
        <Kin<PlainConditions> as Registered>::ENTRY,
        Entry::spelled::<TypeSystem>(Work::Extracted(|model| {
            let extraction = Extraction::default();
            let transitions = Transitions::from(model);
            let system = TypeSystem::built_from(&extraction, &transitions);
            [OneEdge::dirty(&system), DistinctSignatures::dirty(&system)].into_iter().flatten().collect()
        })),
        Entry::spelled::<Clustered<AcyclicQuotients>>(Work::Extracted(|model| {
            <Clustered<AcyclicQuotients> as ModelLint>::report(model).dirty().into_iter().collect()
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

    fn locating(sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Worked {
        Worked::timed(sources.located().timing())
    }

    fn helped(sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>, worker: Worker) -> Worked {
        let readout = ManuallyDrop::new(sources.read(worker));
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
        let mut jobs: Vec<Job> =
            Self::LINTS.into_iter().enumerate().filter_map(|(index, entry)| entry.queued(Slot::from(index))).collect();
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
    Filed(Filing),
    Judged(Judging),
    Ruled(Rules),
    Described(Workspaced),
    Composite(Judging, Workspaced),
}

impl Entry {
    pub(crate) const fn spelled<Lint: ?Sized>(work: Work) -> Entry {
        Entry { name: Named::spelled::<Lint>(), work }
    }

    fn queued(self, slot: Slot) -> Option<Job> {
        match self.work {
            Work::Filed(_) | Work::Judged(_) | Work::Ruled(_) => None,
            Work::Checked(_) | Work::Modelled(_) | Work::Extracted(_) | Work::Described(_) | Work::Composite(..) => {
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
            | Work::Composite(..)
            | Work::Judged(_)
            | Work::Ruled(_) => None,
        }
    }

    fn judging(self) -> Option<Judging> {
        match self.work {
            Work::Judged(judging) | Work::Composite(judging, _) => Some(judging),
            Work::Checked(_)
            | Work::Modelled(_)
            | Work::Extracted(_)
            | Work::Described(_)
            | Work::Filed(_)
            | Work::Ruled(_) => None,
        }
    }

    fn ruling(self) -> Option<Rules> {
        match self.work {
            Work::Ruled(rules) => Some(rules),
            Work::Checked(_)
            | Work::Modelled(_)
            | Work::Extracted(_)
            | Work::Described(_)
            | Work::Composite(..)
            | Work::Filed(_)
            | Work::Judged(_) => None,
        }
    }

    pub(crate) fn headline(&self) -> Option<Headline> {
        let filed = self.filing().map(|filing| filing.headline);
        filed
            .or_else(|| self.judging().map(|judging| judging.headline))
            .or_else(|| self.ruling().map(|rules| rules.headline))
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

impl Work {
    fn awaited<'run>(self, sources: &'run Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Ready<'run> {
        match self {
            Work::Modelled(_) | Work::Extracted(_) => Ready::Modelled(sources.gathering().model()),
            Work::Described(_) | Work::Composite(..) => Ready::Grounded(Ground::awaited(sources)),
            Work::Checked(_) | Work::Filed(_) | Work::Judged(_) | Work::Ruled(_) => Ready::Bare,
        }
    }

    fn reported<'run>(self, readout: &'run Readout, ready: Ready<'run>) -> Vec<Report<'run>> {
        match self {
            Work::Checked(check) => check(readout),
            Work::Modelled(query) => ready.model().map(query).into_iter().collect(),
            Work::Described(_)
            | Work::Composite(..)
            | Work::Extracted(_)
            | Work::Filed(_)
            | Work::Judged(_)
            | Work::Ruled(_) => Vec::new(),
        }
    }

    fn worked<'run>(self, slot: Slot, readout: &'run Readout, ready: Ready<'run>) -> Worked {
        match self {
            Work::Extracted(_) | Work::Described(_) | Work::Composite(..) => self.grounded(slot, ready),
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) | Work::Judged(_) | Work::Ruled(_) => {
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
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) | Work::Judged(_) | Work::Ruled(_) => {
                Worked::default()
            }
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
            Work::Checked(_) | Work::Filed(_) | Work::Judged(_) | Work::Ruled(_) => Rank::Early(Reverse(self.slot)),
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
    concluding: Vec<Concluding<'file>>,
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
        let concluding = Lints::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| Some((Slot::from(index), entry.ruling()?)))
            .filter_map(|(slot, rules)| Some(Concluding { slot, pending: (rules.conclude)(rules.reads.admits(file)?) }))
            .collect();
        Filer { claimed, rulings, concluding, admitted, leads, starts, sorting, finds, sorts }
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
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        let found = self.finds.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() });
        let sorted = found.chain(self.sorts.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }));
        sorted.chain(self.rulings.iter_mut().flat_map(|ruling| ruling.pending.watching()))
    }

    fn gathered(self) -> FileWork<'file> {
        let Filer { claimed, rulings, concluding, admitted, leads, starts, sorting, finds, sorts } = self;
        let file = claimed.file();
        let listed = starts.into_iter().zip(finds).map(|(key, watcher)| (key, vec![watcher.found()]));
        let watched = listed.chain(sorting.into_iter().zip(sorts).map(|(key, watcher)| (key, watcher.sorted())));
        let scanned = leads.iter().filter_map(|(key, lead)| lead.scanned(file).map(|found| (*key, found)));
        let mut sorted: BTreeMap<TypeId, Vec<Vec<Violation<'_>>>> = watched.chain(scanned).collect();
        let found = admitted.into_iter().map(|found| (found.slot, Self::taken(&mut sorted, found.source)));
        let worked = found.map(|(slot, violations)| Worked::filing(slot, claimed.order(), &violations)).collect();
        FileWork { order: claimed.order(), worked, rulings, concluding }
    }
}

impl<Lint: WatchingLint + 'static> Registered for Judged<Lint>
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
    Primed<Lint>: Concluded,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Ruled(Rules { headline: Lint::HEADLINE, reads: Lint::READS, conclude: Self::concluding }),
    };
}

impl<Lint: RuledLint + 'static> Registered for Ruled<Lint>
where
    Ruled<Lint>: Concluded,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
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

#[derive(Debug)]
struct PrimedConclusion<'file, Lint> {
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

impl<'file, Lint: PrimedLint + fmt::Debug> Concludes<'file> for PrimedConclusion<'file, Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Priming<'run, Lint>,
{
    fn concluded<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        Lint::judged(self.file, settled.primer(), settled.primed())
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

impl<Lint: PrimedLint + fmt::Debug + 'static> Concluded for Primed<Lint>
where
    for<'file> PrimedConclusion<'file, Lint>: Concludes<'file>,
{
    fn concluding(file: &SourceFile) -> Box<dyn Concludes<'_> + '_> {
        Box::new(PrimedConclusion::<Lint> { file, lint: PhantomData })
    }
}

trait Judges {
    fn pending(file: &SourceFile) -> Box<dyn Awaits<'_> + '_>;
}

pub(crate) trait Awaits<'file>: fmt::Debug {
    fn watching(&mut self) -> Vec<&mut Watching<'_>>;

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
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watched.watching().into_iter().collect()
    }

    fn judged<'run: 'file>(self: Box<Self>, settled: Settled<'_, 'run>) -> Vec<Violation<'file>> {
        let Waiting { file, watched } = *self;
        Lint::judged(file, watched.gathered(), settled.held())
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

#[derive(Clone, Copy, Debug)]
struct Ground<'run> {
    heads: &'run [FileHead],
    described: &'run Described,
    roles: &'run RoleFacts<'run>,
    model: &'run Model,
}

impl<'run> Ground<'run> {
    fn awaited(sources: &'run Sources<impl Files, impl Cargo, impl Timer + Sync>) -> Option<Ground<'run>> {
        let described = sources.located().done().as_ref().ok()?;
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
    Judged<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Composite(
            Judging { headline: Lint::HEADLINE, reads: Lint::READS, pend: Judged::<Lint>::pending },
            |heads, workspaces, _, model| Lint::report(heads, workspaces, &(), model),
        ),
    };
}
