use code_model::Model;
use code_reader::Reader;
use lint_hand_impls::HandImpls;

use lint_core::{Lint, ModelLint, Readout, Report};

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

use crate::cli::Outcome;
use crate::cli::boundary::{Rendered, Shard, Slot, Workers};

#[derive(Debug)]
pub struct Lints;

type Check = for<'file> fn(&'file Readout) -> Vec<Report<'file>>;

type Query = for<'model> fn(&'model Model) -> Report<'model>;

impl Lints {
    const CHECKS: [Check; 45] = [
        |readout| vec![NoComments::report(readout.files())],
        |readout| vec![NoFreeFns::report(readout.files())],
        |readout| vec![ThinMain::report(readout.files())],
        |readout| vec![VecOutParams::report(readout.files())],
        |readout| vec![NamedLifetimes::report(readout.files())],
        |readout| vec![EnumPredicates::report(readout.files())],
        |readout| vec![ConstShape::report(readout.files())],
        |readout| vec![TypeShape::report(readout.files())],
        |readout| vec![ExpressionFlow::report(readout.files())],
        |readout| vec![FnShape::report(readout.files())],
        |readout| vec![LiteralNames::report(readout.files())],
        |readout| vec![ManualIteration::report(readout.files())],
        |readout| vec![ModuleNesting::report(readout.files())],
        |readout| vec![NamedNumbers::report(readout.files())],
        |readout| vec![NoForwarders::report(readout.files())],
        |readout| vec![NoNumbersInBinaries::report(readout.files())],
        |readout| vec![NoParameterBags::report(readout.files())],
        |readout| vec![NoPlaintext::report(readout.files())],
        |readout| vec![PrimitiveBoundary::report(readout.files())],
        |readout| vec![PrivateFns::report(readout.files())],
        |readout| vec![ProcessStarters::report(readout.files())],
        |readout| vec![RawResources::report(readout.files())],
        |readout| vec![PublicSurface::report(readout.files())],
        |readout| vec![RepeatedMethods::report(readout.files())],
        |readout| vec![StateGraph::report(readout.files())],
        |readout| vec![GuardMustUse::report(readout.files())],
        |readout| vec![CrateGraph::report(readout.files())],
        |readout| vec![PublicApiTests::report(readout.files())],
        |readout| vec![NoTestOnlyApi::report(readout.files())],
        |readout| vec![NoIgnoredTests::report(readout.files())],
        |readout| vec![NoTestIo::report(readout.files())],
        |readout| vec![FakesAtPorts::report(readout.files())],
        |readout| vec![PortBatching::report(readout.files())],
        |readout| vec![LongExactStrings::report(readout.files())],
        |readout| vec![RepeatedShapes::report(readout.files())],
        |readout| vec![MandatoryLaws::report(readout.files())],
        |readout| vec![ArbitraryNewtypes::report(readout.files())],
        |readout| vec![BoolMethods::report(readout.files())],
        |readout| vec![EffectsInPorts::report(readout.files())],
        |readout| vec![NoEarlyExits::report(readout.files())],
        |readout| {
            let flows = Flows::from(readout.files());
            vec![flows.report::<LinearBodies>(), flows.report::<PureRouters>(), flows.report::<PlainConditions>()]
        },
        |readout| {
            let extraction = Extraction::from(readout.files());
            let system = TypeSystem::built_from(readout.files(), &extraction);
            [OneEdge::dirty(&system), <Clustered<AcyclicQuotients>>::dirty(&system), DistinctSignatures::dirty(&system)]
                .into_iter()
                .flatten()
                .collect()
        },
        |readout| vec![UnconsumedTraits::report(readout.files())],
        |readout| vec![RustOnly::report(readout.repo_files())],
        |readout| vec![WorkflowRun::report(readout.repo_files())],
    ];

    const QUERIES: [Query; 1] = [<HandImpls as ModelLint>::report];

    #[must_use]
    pub fn checked(readout: &Readout) -> Outcome<'_> {
        let model = Reader::read(readout);
        let mut rendered: Vec<Rendered> = Self::CHECKS
            .iter()
            .enumerate()
            .flat_map(|(index, check)| {
                check(readout)
                    .into_iter()
                    .filter_map(Report::dirty)
                    .map(move |report| Rendered::render(Slot::Check(index), &report))
            })
            .collect();
        rendered
            .extend(Workers::available().at_most(Self::QUERIES.len()).spread(|shard| Self::answered(&model, shard)));
        rendered.sort_by_key(Rendered::slot);
        Outcome::from_rendered(rendered, readout.failures())
    }

    fn answered(model: &Model, shard: Shard) -> Vec<Rendered> {
        shard
            .picks(&Self::QUERIES)
            .filter_map(|(index, query)| {
                query(model).dirty().map(|report| Rendered::render(Slot::Query(index), &report))
            })
            .collect()
    }
}
