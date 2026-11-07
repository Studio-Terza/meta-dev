use std::iter;
use std::thread::Scope;

use code_model::Model;

use lint_hand_impls::HandImpls;

use file_ports::Files;
use lint_core::{Lint, ModelLint, Readout, Report};
use lint_reader::SourceFiles;

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
use lint_thin_main::ThinMain;
use lint_transition_graph::{AcyclicQuotients, Clustered, Extraction, GraphLint, OneEdge, TypeSystem};
use lint_type_shape::TypeShape;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_vec_out_params::VecOutParams;
use lint_workflow_run::WorkflowRun;

use crate::cli::boundary::{Helper, Helpers, Queue, Rendered, Slot, Workers};
use crate::cli::{Outcome, Sources};

#[derive(Debug)]
pub struct Lints;

type Check = for<'file> fn(&'file Readout, &'file Queue) -> Vec<Report<'file>>;

type Query = for<'model> fn(&'model Model) -> Report<'model>;

impl Lints {
    const CHECKS: [Check; 44] = [
        |readout, _| vec![NoComments::report(readout.files())],
        |readout, _| vec![NoFreeFns::report(readout.files())],
        |readout, _| vec![ThinMain::report(readout.files())],
        |readout, _| vec![VecOutParams::report(readout.files())],
        |readout, _| vec![NamedLifetimes::report(readout.files())],
        |readout, _| vec![EnumPredicates::report(readout.files())],
        |readout, _| vec![ConstShape::report(readout.files())],
        |readout, _| vec![TypeShape::report(readout.files())],
        |readout, _| vec![ExpressionFlow::report(readout.files())],
        |readout, _| vec![FnShape::report(readout.files())],
        |readout, _| vec![LiteralNames::report(readout.files())],
        |readout, _| vec![ManualIteration::report(readout.files())],
        |readout, _| vec![ModuleNesting::report(readout.files())],
        |readout, _| vec![NamedNumbers::report(readout.files())],
        |readout, _| vec![NoForwarders::report(readout.files())],
        |readout, _| vec![NoNumbersInBinaries::report(readout.files())],
        |readout, _| vec![NoParameterBags::report(readout.files())],
        |readout, queue| vec![<PrimitiveBoundary as ModelLint>::report(queue.model(readout))],
        |readout, _| vec![PrivateFns::report(readout.files())],
        |readout, _| vec![ProcessStarters::report(readout.files())],
        |readout, _| vec![RawResources::report(readout.files())],
        |readout, _| vec![PublicSurface::report(readout.files())],
        |readout, _| vec![RepeatedMethods::report(readout.files())],
        |readout, _| vec![StateGraph::report(readout.files())],
        |readout, _| vec![GuardMustUse::report(readout.files())],
        |readout, _| vec![CrateGraph::report(readout.files())],
        |readout, _| vec![PublicApiTests::report(readout.files())],
        |readout, _| vec![NoTestOnlyApi::report(readout.files())],
        |readout, _| vec![NoIgnoredTests::report(readout.files())],
        |readout, _| vec![NoTestIo::report(readout.files())],
        |readout, _| vec![FakesAtPorts::report(readout.files())],
        |readout, _| vec![PortBatching::report(readout.files())],
        |readout, _| vec![LongExactStrings::report(readout.files())],
        |readout, _| vec![RepeatedShapes::report(readout.files())],
        |readout, _| vec![MandatoryLaws::report(readout.files())],
        |readout, _| vec![ArbitraryNewtypes::report(readout.files())],
        |readout, _| vec![BoolMethods::report(readout.files())],
        |readout, _| vec![EffectsInPorts::report(readout.files())],
        |readout, _| vec![NoEarlyExits::report(readout.files())],
        |readout, _| {
            let flows = Flows::from(readout.files());
            vec![flows.report::<LinearBodies>(), flows.report::<PureRouters>(), flows.report::<PlainConditions>()]
        },
        |readout, _| {
            let extraction = Extraction::from(readout.files());
            let system = TypeSystem::built_from(readout.files(), &extraction);
            [OneEdge::dirty(&system), <Clustered<AcyclicQuotients>>::dirty(&system), DistinctSignatures::dirty(&system)]
                .into_iter()
                .flatten()
                .collect()
        },
        |readout, _| vec![UnconsumedTraits::report(readout.files())],
        |readout, _| vec![RustOnly::report(readout.repo_files())],
        |readout, _| vec![WorkflowRun::report(readout.repo_files())],
    ];

    const QUERIES: [Query; 2] = [<HandImpls as ModelLint>::report, <NoPlaintext as ModelLint>::report];

    pub fn started<'scope, F: Files + Sync>(
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F>,
    ) -> Helpers<'scope> {
        Workers::available()
            .helpers()
            .filter_map(|_| Helper::spawned(scope, move || Self::helped(sources)).ok())
            .collect()
    }

    #[must_use]
    pub fn checked<'file>(readout: &'file Readout, queue: &Queue, helpers: Helpers<'_>) -> Outcome<'file> {
        let mine = Self::worked(readout, queue);
        let mut rendered: Vec<Rendered> =
            iter::once(mine).chain(helpers.into_iter().map(Helper::joined)).flatten().collect();
        rendered.sort_by_key(Rendered::slot);
        Outcome::from_rendered(rendered, readout.failures())
    }

    fn helped(sources: &Sources<impl Files>) -> Vec<Rendered> {
        Self::worked(&SourceFiles::read_staged(sources.files(), sources.stage()), sources.queue())
    }

    fn worked(readout: &Readout, queue: &Queue) -> Vec<Rendered> {
        let jobs: Vec<Job> = Self::CHECKS
            .into_iter()
            .enumerate()
            .map(|(index, check)| Job::Check { slot: Slot::Check(index), check })
            .chain(
                Self::QUERIES
                    .into_iter()
                    .enumerate()
                    .map(|(index, query)| Job::Query { slot: Slot::Query(index), query }),
            )
            .rev()
            .collect();
        queue.claimed(&jobs).flat_map(|job| job.done(readout, queue)).collect()
    }
}

#[derive(Clone, Copy, Debug)]
enum Job {
    Check { slot: Slot, check: Check },
    Query { slot: Slot, query: Query },
}

impl Job {
    fn done(self, readout: &Readout, queue: &Queue) -> Vec<Rendered> {
        match self {
            Job::Check { slot, check } => check(readout, queue)
                .into_iter()
                .filter_map(Report::dirty)
                .map(|report| Rendered::render(slot, &report))
                .collect(),
            Job::Query { slot, query } => {
                query(queue.model(readout)).dirty().map(|report| Rendered::render(slot, &report)).into_iter().collect()
            }
        }
    }
}
