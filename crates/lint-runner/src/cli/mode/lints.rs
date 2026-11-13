use std::cmp::Reverse;
use std::io::Write;
use std::thread::Scope;

use cargo_ports::Cargo;
use code_model::Model;

use lint_hand_impls::HandImpls;

use file_ports::Files;
use lint_core::{Lint, ModelLint, Readout, Report, SourceFile, WorkspaceLint};
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
use lint_linear_flow::{Flows, LinearBodies, PlainConditions, PureRouters};
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
use lint_transition_graph::{AcyclicQuotients, Clustered, Extraction, GraphLint, OneEdge, TypeSystem};
use lint_type_shape::TypeShape;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_vec_out_params::VecOutParams;
use lint_workflow_run::WorkflowRun;

use super::Survey;
use crate::cli::boundary::{Helper, Helpers, Rendered, Slot, Surveyed, Worked, Workers};
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

type Query = for<'model> fn(&'model Model) -> Report<'model>;

type Workspaced = for<'file> fn(&'file [SourceFile], &'file [Metadata]) -> Report<'file>;

impl Lints {
    const LINTS: [Entry; 51] = [
        Entry::spelled::<NoComments>(Work::Checked(|readout| vec![NoComments::report(readout.files())])),
        Entry::spelled::<NoFreeFns>(Work::Checked(|readout| vec![NoFreeFns::report(readout.files())])),
        Entry::spelled::<ThinMain>(Work::Checked(|readout| vec![ThinMain::report(readout.files())])),
        Entry::spelled::<VecOutParams>(Work::Checked(|readout| vec![VecOutParams::report(readout.files())])),
        Entry::spelled::<NamedLifetimes>(Work::Checked(|readout| vec![NamedLifetimes::report(readout.files())])),
        Entry::spelled::<EnumPredicates>(Work::Checked(|readout| vec![EnumPredicates::report(readout.files())])),
        Entry::spelled::<ConstShape>(Work::Checked(|readout| vec![ConstShape::report(readout.files())])),
        Entry::spelled::<TypeShape>(Work::Checked(|readout| vec![TypeShape::report(readout.files())])),
        Entry::spelled::<ExpressionFlow>(Work::Checked(|readout| vec![ExpressionFlow::report(readout.files())])),
        Entry::spelled::<FnShape>(Work::Checked(|readout| vec![FnShape::report(readout.files())])),
        Entry::spelled::<LiteralNames>(Work::Checked(|readout| vec![LiteralNames::report(readout.files())])),
        Entry::spelled::<ManualIteration>(Work::Checked(|readout| vec![ManualIteration::report(readout.files())])),
        Entry::spelled::<ModuleNesting>(Work::Checked(|readout| vec![ModuleNesting::report(readout.files())])),
        Entry::spelled::<NamedNumbers>(Work::Checked(|readout| vec![NamedNumbers::report(readout.files())])),
        Entry::spelled::<NoForwarders>(Work::Checked(|readout| vec![NoForwarders::report(readout.files())])),
        Entry::spelled::<NoNumbersInBinaries>(Work::Checked(|readout| {
            vec![NoNumbersInBinaries::report(readout.files())]
        })),
        Entry::spelled::<NoParameterBags>(Work::Checked(|readout| vec![NoParameterBags::report(readout.files())])),
        Entry::spelled::<PrimitiveBoundary>(Work::Modelled(<PrimitiveBoundary as ModelLint>::report)),
        Entry::spelled::<PrivateFns>(Work::Checked(|readout| vec![PrivateFns::report(readout.files())])),
        Entry::spelled::<ProcessStarters>(Work::Checked(|readout| vec![ProcessStarters::report(readout.files())])),
        Entry::spelled::<RawResources>(Work::Checked(|readout| vec![RawResources::report(readout.files())])),
        Entry::spelled::<PublicSurface>(Work::Checked(|readout| vec![PublicSurface::report(readout.files())])),
        Entry::spelled::<RepeatedMethods>(Work::Checked(|readout| vec![RepeatedMethods::report(readout.files())])),
        Entry::spelled::<StateGraph>(Work::Checked(|readout| vec![StateGraph::report(readout.files())])),
        Entry::spelled::<GuardMustUse>(Work::Checked(|readout| vec![GuardMustUse::report(readout.files())])),
        Entry::spelled::<PublicApiTests>(Work::Checked(|readout| vec![PublicApiTests::report(readout.files())])),
        Entry::spelled::<NoTestOnlyApi>(Work::Checked(|readout| vec![NoTestOnlyApi::report(readout.files())])),
        Entry::spelled::<NoIgnoredTests>(Work::Checked(|readout| vec![NoIgnoredTests::report(readout.files())])),
        Entry::spelled::<NoTestIo>(Work::Checked(|readout| vec![NoTestIo::report(readout.files())])),
        Entry::spelled::<FakesAtPorts>(Work::Checked(|readout| vec![FakesAtPorts::report(readout.files())])),
        Entry::spelled::<PortBatching>(Work::Checked(|readout| vec![PortBatching::report(readout.files())])),
        Entry::spelled::<OwnedDependencies>(Work::Described(<OwnedDependencies as WorkspaceLint>::report)),
        Entry::spelled::<KeyDependencyRoles>(Work::Described(<KeyDependencyRoles as WorkspaceLint>::report)),
        Entry::spelled::<EffectsInFillers>(Work::Checked(|readout| vec![EffectsInFillers::report(readout.files())])),
        Entry::spelled::<CountedCalls>(Work::Checked(|readout| vec![CountedCalls::report(readout.files())])),
        Entry::spelled::<HiddenSqlx>(Work::Checked(|readout| vec![HiddenSqlx::report(readout.files())])),
        Entry::spelled::<StoreQueries>(Work::Checked(|readout| vec![StoreQueries::report(readout.files())])),
        Entry::spelled::<LongExactStrings>(Work::Checked(|readout| vec![LongExactStrings::report(readout.files())])),
        Entry::spelled::<RepeatedShapes>(Work::Checked(|readout| vec![RepeatedShapes::report(readout.files())])),
        Entry::spelled::<MandatoryLaws>(Work::Checked(|readout| vec![MandatoryLaws::report(readout.files())])),
        Entry::spelled::<ArbitraryNewtypes>(Work::Checked(|readout| vec![ArbitraryNewtypes::report(readout.files())])),
        Entry::spelled::<BoolMethods>(Work::Checked(|readout| vec![BoolMethods::report(readout.files())])),
        Entry::spelled::<EffectsInPorts>(Work::Checked(|readout| vec![EffectsInPorts::report(readout.files())])),
        Entry::spelled::<NoEarlyExits>(Work::Checked(|readout| vec![NoEarlyExits::report(readout.files())])),
        Entry::spelled::<Flows>(Work::Checked(|readout| {
            let flows = Flows::from(readout.files());
            vec![flows.report::<LinearBodies>(), flows.report::<PureRouters>(), flows.report::<PlainConditions>()]
        })),
        Entry::spelled::<TypeSystem>(Work::Checked(|readout| {
            let extraction = Extraction::from(readout.files());
            let system = TypeSystem::built_from(readout.files(), &extraction);
            [OneEdge::dirty(&system), <Clustered<AcyclicQuotients>>::dirty(&system), DistinctSignatures::dirty(&system)]
                .into_iter()
                .flatten()
                .chain([CrateGraph::of_names(extraction.names())])
                .collect()
        })),
        Entry::spelled::<UnconsumedTraits>(Work::Checked(|readout| vec![UnconsumedTraits::report(readout.files())])),
        Entry::spelled::<RustOnly>(Work::Checked(|readout| vec![RustOnly::report(readout.repo_files())])),
        Entry::spelled::<WorkflowRun>(Work::Checked(|readout| vec![WorkflowRun::report(readout.repo_files())])),
        Entry::spelled::<HandImpls>(Work::Modelled(<HandImpls as ModelLint>::report)),
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
        let mut rendered = worked.into_rendered();
        rendered.sort_by_key(Rendered::slot);
        unlocated.map_or_else(|| Outcome::from_rendered(rendered, readout.failures()), Outcome::Unlocated)
    }

    fn helped(sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let readout = sources.read(worker);
        [Self::worked(readout.done(), sources, worker), Worked::timed(readout.timing())].into_iter().collect()
    }

    fn worked(readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>, worker: Worker) -> Worked {
        let lowered = sources.lowered(readout, worker);
        let mut jobs: Vec<Job> = Self::LINTS
            .into_iter()
            .enumerate()
            .map(|(index, entry)| entry.job(Slot::from(index)))
            .chain([Job::SURVEY])
            .collect();
        jobs.sort_by_key(Job::rank);
        let claimed = sources.queue().claimed(&jobs).map(|job| job.done(readout, sources));
        lowered.into_iter().map(Worked::timed).chain(claimed).collect()
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
    Described(Workspaced),
    Survey,
}

impl Entry {
    const fn spelled<Lint: ?Sized>(work: Work) -> Entry {
        Entry { name: Named::spelled::<Lint>(), work }
    }

    fn job(self, slot: Slot) -> Job {
        Job { slot, step: Step::Lint(self.name), work: self.work }
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
            Work::Modelled(query) => sources.model().map(query).into_iter().collect(),
            Work::Described(described) => Self::described(described, readout, sources),
            Work::Survey => Vec::new(),
        }
    }

    fn worked(self, slot: Slot, readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        match self {
            Work::Survey => Worked::surveying(&Survey::measured(readout.files())),
            Work::Checked(_) | Work::Modelled(_) | Work::Described(_) => {
                Worked::rendering(slot, self.reported(readout, sources))
            }
        }
    }

    fn described<'run>(
        described: Workspaced,
        readout: &'run Readout,
        sources: &'run Sources<impl Files, impl Cargo, impl Timer>,
    ) -> Vec<Report<'run>> {
        let workspaces = sources.located().done().as_ref().ok().map(Described::workspaces);
        workspaces.map(|found| described(readout.files(), found)).into_iter().collect()
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
            Work::Checked(_) => Rank::Early(Reverse(self.slot)),
            Work::Modelled(_) | Work::Described(_) => Rank::Late(self.slot),
        }
    }

    fn done(self, readout: &Readout, sources: &Sources<impl Files, impl Cargo, impl Timer>) -> Worked {
        let stopwatch = Stopwatch::started(sources.wired().timer());
        let worked = self.work.worked(self.slot, readout, sources);
        [worked, Worked::timed(stopwatch.stopped(self.step))].into_iter().collect()
    }
}
