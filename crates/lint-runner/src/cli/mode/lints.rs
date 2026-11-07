use std::iter;
use std::thread::Scope;

use code_model::Model;
use code_reader::Reader;

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

type Check = for<'file> fn(&'file Readout) -> Vec<Report<'file>>;

type Query = for<'model> fn(&'model Model) -> Report<'model>;

impl Lints {
    const LINTS: [Entry; 46] = [
        Entry::Checked(|readout| vec![NoComments::report(readout.files())]),
        Entry::Checked(|readout| vec![NoFreeFns::report(readout.files())]),
        Entry::Checked(|readout| vec![ThinMain::report(readout.files())]),
        Entry::Checked(|readout| vec![VecOutParams::report(readout.files())]),
        Entry::Checked(|readout| vec![NamedLifetimes::report(readout.files())]),
        Entry::Checked(|readout| vec![EnumPredicates::report(readout.files())]),
        Entry::Checked(|readout| vec![ConstShape::report(readout.files())]),
        Entry::Checked(|readout| vec![TypeShape::report(readout.files())]),
        Entry::Checked(|readout| vec![ExpressionFlow::report(readout.files())]),
        Entry::Checked(|readout| vec![FnShape::report(readout.files())]),
        Entry::Checked(|readout| vec![LiteralNames::report(readout.files())]),
        Entry::Checked(|readout| vec![ManualIteration::report(readout.files())]),
        Entry::Checked(|readout| vec![ModuleNesting::report(readout.files())]),
        Entry::Checked(|readout| vec![NamedNumbers::report(readout.files())]),
        Entry::Checked(|readout| vec![NoForwarders::report(readout.files())]),
        Entry::Checked(|readout| vec![NoNumbersInBinaries::report(readout.files())]),
        Entry::Checked(|readout| vec![NoParameterBags::report(readout.files())]),
        Entry::Modelled(<PrimitiveBoundary as ModelLint>::report),
        Entry::Checked(|readout| vec![PrivateFns::report(readout.files())]),
        Entry::Checked(|readout| vec![ProcessStarters::report(readout.files())]),
        Entry::Checked(|readout| vec![RawResources::report(readout.files())]),
        Entry::Checked(|readout| vec![PublicSurface::report(readout.files())]),
        Entry::Checked(|readout| vec![RepeatedMethods::report(readout.files())]),
        Entry::Checked(|readout| vec![StateGraph::report(readout.files())]),
        Entry::Checked(|readout| vec![GuardMustUse::report(readout.files())]),
        Entry::Checked(|readout| vec![CrateGraph::report(readout.files())]),
        Entry::Checked(|readout| vec![PublicApiTests::report(readout.files())]),
        Entry::Checked(|readout| vec![NoTestOnlyApi::report(readout.files())]),
        Entry::Checked(|readout| vec![NoIgnoredTests::report(readout.files())]),
        Entry::Checked(|readout| vec![NoTestIo::report(readout.files())]),
        Entry::Checked(|readout| vec![FakesAtPorts::report(readout.files())]),
        Entry::Checked(|readout| vec![PortBatching::report(readout.files())]),
        Entry::Checked(|readout| vec![LongExactStrings::report(readout.files())]),
        Entry::Checked(|readout| vec![RepeatedShapes::report(readout.files())]),
        Entry::Checked(|readout| vec![MandatoryLaws::report(readout.files())]),
        Entry::Checked(|readout| vec![ArbitraryNewtypes::report(readout.files())]),
        Entry::Checked(|readout| vec![BoolMethods::report(readout.files())]),
        Entry::Checked(|readout| vec![EffectsInPorts::report(readout.files())]),
        Entry::Checked(|readout| vec![NoEarlyExits::report(readout.files())]),
        Entry::Checked(|readout| {
            let flows = Flows::from(readout.files());
            vec![flows.report::<LinearBodies>(), flows.report::<PureRouters>(), flows.report::<PlainConditions>()]
        }),
        Entry::Checked(|readout| {
            let extraction = Extraction::from(readout.files());
            let system = TypeSystem::built_from(readout.files(), &extraction);
            [OneEdge::dirty(&system), <Clustered<AcyclicQuotients>>::dirty(&system), DistinctSignatures::dirty(&system)]
                .into_iter()
                .flatten()
                .collect()
        }),
        Entry::Checked(|readout| vec![UnconsumedTraits::report(readout.files())]),
        Entry::Checked(|readout| vec![RustOnly::report(readout.repo_files())]),
        Entry::Checked(|readout| vec![WorkflowRun::report(readout.repo_files())]),
        Entry::Modelled(<HandImpls as ModelLint>::report),
        Entry::Modelled(<NoPlaintext as ModelLint>::report),
    ];

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
        let jobs: Vec<Job> = Self::LINTS
            .into_iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.checked().map(|check| Job::Check { slot: Slot::from(index), check }))
            .chain(iter::once(Job::Modelled))
            .rev()
            .collect();
        queue.claimed(&jobs).flat_map(|job| job.done(readout)).collect()
    }

    fn modelled(readout: &Readout) -> Vec<Rendered> {
        let model = Reader::read(readout);
        Self::LINTS
            .into_iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                entry
                    .modelled()
                    .and_then(|query| query(&model).dirty())
                    .map(|report| Rendered::render(Slot::from(index), &report))
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
enum Entry {
    Checked(Check),
    Modelled(Query),
}

impl Entry {
    fn checked(self) -> Option<Check> {
        match self {
            Entry::Checked(check) => Some(check),
            Entry::Modelled(_) => None,
        }
    }

    fn modelled(self) -> Option<Query> {
        match self {
            Entry::Modelled(query) => Some(query),
            Entry::Checked(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Job {
    Check { slot: Slot, check: Check },
    Modelled,
}

impl Job {
    fn done(self, readout: &Readout) -> Vec<Rendered> {
        match self {
            Job::Check { slot, check } => check(readout)
                .into_iter()
                .filter_map(Report::dirty)
                .map(|report| Rendered::render(slot, &report))
                .collect(),
            Job::Modelled => Lints::modelled(readout),
        }
    }
}
