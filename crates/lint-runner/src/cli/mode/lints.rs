use std::cmp::Reverse;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::thread::Scope;

use cargo_ports::Cargo;
use code_model::Model;

use lint_hand_impls::HandImpls;

use file_ports::Files;
use lint_core::{FileHead, Headline, Lint, ModelLint, Readout, Report, SourceFile, Violation, WorkspaceLint};
use lint_file::{ContextLint, FileLint, Filed, Judged, Primed, PrimedLint, Reads};
use lint_port_dependencies::{RoleFacts, RoleReading};
use lint_reader::{Described, Unlocated};
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
use lint_port_batching::PortBatching;
use lint_port_dependencies::{CountedCalls, EffectsInFillers, HiddenSqlx, KeyDependencyRoles, OwnedDependencies};
use lint_primitive_boundary::PrimitiveBoundary;
use lint_private_fns::PrivateFns;
use lint_process_starters::ProcessStarters;
use lint_public_api_tests::PublicApiTests;
use lint_public_surface::PublicSurface;
use lint_raw_resources::RawResources;
use lint_repeated_methods::RepeatedMethods;
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
use crate::cli::boundary::{Helper, Helpers, Rendered, Slot, Surveyed, Worked, Workers};
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

type Scan = for<'file> fn(&'file SourceFile) -> Vec<Violation<'file>>;

type Judge = for<'file, 'settled, 'run> fn(&'file SourceFile, &'file Settled<'settled, 'run>) -> Vec<Violation<'file>>;

#[derive(Clone, Copy, Debug)]
struct Judging {
    headline: Headline,
    reads: Reads,
    judge: Judge,
}

#[derive(Clone, Copy, Debug)]
struct Filing {
    headline: Headline,
    reads: Reads,
    scan: Scan,
}

type Query = for<'model> fn(&'model Model) -> Report<'model>;

type Workspaced =
    for<'file, 'head> fn(&'file [FileHead], &'file [Metadata], &'file RoleFacts<'head>, &'file Model) -> Report<'file>;

type Extracting = for<'file> fn(&'file Extraction<'file>, &'file Model) -> Vec<Report<'file>>;

impl Lints {
    const LINTS: [Entry; 54] = [
        <Filed<NoComments> as Registered>::ENTRY,
        <Filed<NoFreeFns> as Registered>::ENTRY,
        <Filed<ThinMain> as Registered>::ENTRY,
        <Filed<VecOutParams<'static>> as Registered>::ENTRY,
        <Filed<NamedLifetimes> as Registered>::ENTRY,
        <Judged<EnumPredicates> as Registered>::ENTRY,
        <Filed<ConstShape> as Registered>::ENTRY,
        <Filed<TypeShape> as Registered>::ENTRY,
        <Filed<ExpressionFlow> as Registered>::ENTRY,
        <Filed<FnShape> as Registered>::ENTRY,
        <Filed<LiteralNames> as Registered>::ENTRY,
        <Filed<ManualIteration> as Registered>::ENTRY,
        <Judged<ModuleNesting> as Registered>::ENTRY,
        <Judged<NamedNumbers> as Registered>::ENTRY,
        <Judged<NoForwarders> as Registered>::ENTRY,
        <Filed<NoNumbersInBinaries> as Registered>::ENTRY,
        <Judged<NoParameterBags> as Registered>::ENTRY,
        Entry::spelled::<PrimitiveBoundary>(Work::Modelled(<PrimitiveBoundary as ModelLint>::report)),
        <Filed<PrivateFns> as Registered>::ENTRY,
        Entry::spelled::<ProcessStarters>(Work::Modelled(<ProcessStarters as ModelLint>::report)),
        Entry::spelled::<RawResources>(Work::Modelled(<RawResources as ModelLint>::report)),
        <Judged<PublicSurface> as Registered>::ENTRY,
        <Judged<RepeatedMethods> as Registered>::ENTRY,
        <Primed<StateGraph> as Registered>::ENTRY,
        <Judged<GuardMustUse> as Registered>::ENTRY,
        <Judged<PublicApiTests> as Registered>::ENTRY,
        <Judged<NoTestOnlyApi> as Registered>::ENTRY,
        <Filed<NoIgnoredTests> as Registered>::ENTRY,
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
        <Filed<CountedCalls> as Registered>::ENTRY,
        <Filed<HiddenSqlx> as Registered>::ENTRY,
        Entry::spelled::<StoreQueries>(Work::Modelled(<StoreQueries as ModelLint>::report)),
        <Filed<LongExactStrings> as Registered>::ENTRY,
        <Judged<RepeatedShapes> as Registered>::ENTRY,
        Entry::spelled::<MandatoryLaws>(Work::Modelled(<MandatoryLaws as ModelLint>::report)),
        <Filed<ArbitraryNewtypes> as Registered>::ENTRY,
        <Filed<BoolMethods> as Registered>::ENTRY,
        Entry::spelled::<EffectsInPorts>(Work::Modelled(<EffectsInPorts as ModelLint>::report)),
        <Filed<NoEarlyExits> as Registered>::ENTRY,
        <Filed<LinearBodies> as Registered>::ENTRY,
        <Filed<PureRouters> as Registered>::ENTRY,
        <Filed<PlainConditions> as Registered>::ENTRY,
        Entry::spelled::<TypeSystem>(Work::Extracted(|extraction, model| {
            let transitions = Transitions::from(model);
            let system = TypeSystem::built_from(extraction, &transitions);
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
        Crew { helpers: helpers.collect() }
    }

    fn gathered<'run>(readout: &'run Readout, unlocated: Option<&'run Unlocated>, worked: Worked) -> Outcome<'run> {
        let mut rendered = worked.into_rendered(|slot| slot.picked(&Self::LINTS).and_then(Entry::headline));
        rendered.sort_by_key(Rendered::slot);
        unlocated.map_or_else(|| Outcome::from_rendered(rendered, readout.failures()), Outcome::Unlocated)
    }

    fn helped(sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let readout = ManuallyDrop::new(sources.read(worker));
        [Self::worked(readout.done(), sources, worker), Worked::timed(readout.timing())].into_iter().collect()
    }

    fn worked(readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let lowered = sources.gathered(readout, worker);
        let filing = Stopwatch::started(sources.wired().timer());
        let filed: Worked = Self::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.filing().map(|found| found.lines(Slot::from(index), lowered.claimed())))
            .collect();
        let filed_timing = filing.stopped(Step::Filed(worker));
        let priming = Stopwatch::started(sources.wired().timer());
        sources.gathering().primed(lowered.claimed());
        let primed_timing = priming.stopped(Step::Primed(worker));
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
        let judged: Worked = Self::LINTS
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                entry.judging().map(|found| found.lines(Slot::from(index), lowered.claimed(), &settled))
            })
            .collect();
        let judged_timing = judging.stopped(Step::Judged(worker));
        let timings = lowered.timings().iter().copied().chain([filed_timing, primed_timing, judged_timing]);
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

impl Judging {
    fn lines(self, slot: Slot, claimed: &[Claimed<'_>], settled: &Settled<'_, '_>) -> Worked {
        let read = claimed.iter().filter_map(|found| self.reads.admits(found.file()).map(|file| (found.order(), file)));
        read.map(|(order, file)| Worked::filing(slot, order, &(self.judge)(file, settled))).collect()
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
            Work::Extracted(_) | Work::Described(_) => {
                self.grounded(slot, sources, &Settled::from(sources.gathering()))
            }
            Work::Checked(_) | Work::Modelled(_) | Work::Filed(_) | Work::Judged(_) => Worked::default(),
        }
    }

    fn grounded(
        self,
        slot: Slot,
        sources: &Sources<impl Files, impl Cargo, impl Timer>,
        settled: &Settled<'_, '_>,
    ) -> Worked {
        match self {
            Work::Extracted(extracted) => Worked::rendering(
                slot,
                sources.gathering().model().map_or_default(|model| extracted(sources.gathering().extraction(), model)),
            ),
            Work::Described(described) => Worked::rendering(
                slot,
                sources
                    .located()
                    .done()
                    .as_ref()
                    .ok()
                    .zip(sources.gathering().model())
                    .map(|(located, model)| {
                        let facts = Priming::<RoleReading>::primed(settled);
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

impl Filing {
    fn lines(self, slot: Slot, claimed: &[Claimed<'_>]) -> Worked {
        let read = claimed.iter().filter_map(|found| self.reads.admits(found.file()).map(|file| (found.order(), file)));
        read.map(|(order, file)| Worked::filing(slot, order, &(self.scan)(file))).collect()
    }
}

trait Registered {
    const ENTRY: Entry;
}

impl<Lint: FileLint> Registered for Filed<Lint> {
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Filed(Filing { headline: Lint::HEADLINE, reads: Lint::READS, scan: Lint::violations }),
    };
}

impl<Lint: ContextLint> Registered for Judged<Lint>
where
    Judged<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Judged(Judging { headline: Lint::HEADLINE, reads: Lint::READS, judge: Self::judged_by }),
    };
}

impl<Lint: PrimedLint> Registered for Primed<Lint>
where
    Primed<Lint>: Judges,
{
    const ENTRY: Entry = Entry {
        name: Named::spelled::<Lint>(),
        work: Work::Judged(Judging { headline: Lint::HEADLINE, reads: Lint::READS, judge: Self::judged_by }),
    };
}

trait Judges {
    fn judged_by<'file>(file: &'file SourceFile, settled: &'file Settled<'_, '_>) -> Vec<Violation<'file>>;
}

impl<Lint: ContextLint> Judges for Judged<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Holding<'run, Lint>,
{
    fn judged_by<'file>(file: &'file SourceFile, settled: &'file Settled<'_, '_>) -> Vec<Violation<'file>> {
        Lint::judged(file, settled.held())
    }
}

impl<Lint: PrimedLint> Judges for Primed<Lint>
where
    for<'settled, 'run> Settled<'settled, 'run>: Priming<'run, Lint>,
{
    fn judged_by<'file>(file: &'file SourceFile, settled: &'file Settled<'_, '_>) -> Vec<Violation<'file>> {
        Lint::judged(file, settled.primer(), settled.primed())
    }
}
