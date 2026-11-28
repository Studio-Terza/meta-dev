mod bool_methods;
mod expression_flow;
mod manual_iteration;
mod module_cycles;
mod named_lifetimes;
mod no_comments;
mod no_dyn;
mod no_free_fns;
mod no_test_only_api;
mod process_starters;
mod public_api_tests;
mod public_surface;
mod type_shape;
mod vec_out_params;

use std::sync::Arc;

use crate::budget::{Budget, TestScan};
use crate::finding::Finding;
use crate::gate::Gate;
use crate::graph::Graph;
use crate::source::Source;
use crate::style::Style;
use crate::walk::{Lineup, Pausable, Walker};
use crate::workspace::Package;
pub(crate) use bool_methods::BoolMethods;
pub(crate) use expression_flow::ExpressionFlow;
pub(crate) use manual_iteration::ManualIteration;
pub(crate) use module_cycles::{ModuleCycles, ModuleFact};
pub(crate) use named_lifetimes::NamedLifetimes;
use no_comments::NoComments;
pub(crate) use no_dyn::NoDyn;
pub(crate) use no_free_fns::NoFreeFns;
pub(crate) use no_test_only_api::NoTestOnlyApi;
pub(crate) use process_starters::ProcessStarters;
pub(crate) use public_api_tests::{ApiFact, Emitters, PublicApiTests};
pub(crate) use public_surface::{PublicSurface, SurfaceFact};
pub(crate) use type_shape::TypeShape;
pub(crate) use vec_out_params::VecOutParams;

#[derive(Debug, Default)]
pub(crate) struct Facts {
    surface: Vec<SurfaceFact>,
    api: Vec<ApiFact>,
    test_only: Vec<Finding>,
    modules: Vec<ModuleFact>,
}

#[derive(Debug, Default)]
pub(crate) struct Linted {
    pub found: Vec<Finding>,
    pub facts: Facts,
    pub budget: Budget,
}

#[derive(Debug)]
pub(crate) struct Lints<'run> {
    source: &'run Source,
    entered: Option<()>,
}

impl<'run> Lints<'run> {
    pub(crate) fn linted(source: &'run Source, file: &syn::File, gate: Gate) -> Linted {
        let lints = Lints { source, entered: gate.entered() };
        let lineup = Lineup {
            no_free_fns: lints.style(),
            vec_out_params: lints.style(),
            named_lifetimes: lints.style(),
            type_shape: lints.style(),
            expression_flow: lints.style(),
            manual_iteration: lints.style(),
            process_starters: lints.style(),
            bool_methods: lints.style(),
            no_dyn: lints.style(),
            public_surface: lints.graph(),
            public_api_tests: lints.graph(),
            emitters: Pausable::admitted(Emitters::default(), lints.entered),
            no_test_only_api: lints.graph(),
            test_scan: Pausable::admitted(TestScan::default(), Some(())),
            module_cycles: Pausable::admitted(ModuleCycles::new(source), Some(())),
        };
        let Lineup {
            no_free_fns,
            vec_out_params,
            named_lifetimes,
            type_shape,
            expression_flow,
            manual_iteration,
            process_starters,
            bool_methods,
            no_dyn,
            public_surface,
            public_api_tests,
            emitters,
            no_test_only_api,
            test_scan,
            module_cycles,
        } = Walker::walked(lineup, file);
        let found = [
            lints.found(lints.style::<NoComments>()),
            lints.found(no_free_fns),
            lints.found(vec_out_params),
            lints.found(named_lifetimes),
            lints.found(type_shape),
            lints.found(expression_flow),
            lints.found(manual_iteration),
            lints.found(process_starters),
            lints.found(bool_methods),
            lints.found(no_dyn),
        ];
        let api = Lints::facts(public_api_tests).into_iter().chain(emitters.woken().map_or_default(Emitters::facts));
        let facts = Facts {
            surface: Lints::facts(public_surface),
            api: api.collect(),
            test_only: Lints::facts(no_test_only_api),
            modules: Lints::facts(module_cycles),
        };
        let budget = Budget::of_file(source, test_scan.woken().unwrap_or_default(), gate);
        Linted { found: found.into_iter().flatten().collect(), facts, budget }
    }

    fn style<Lint: Style<'run>>(&self) -> Pausable<Lint> {
        Pausable::admitted(Lint::new(self.source), self.entered.and(Lint::READS.admits(self.source)))
    }

    fn graph<Lint: Graph<'run>>(&self) -> Pausable<Lint> {
        Pausable::admitted(Lint::new(self.source), self.entered)
    }

    fn found<Lint: Style<'run>>(&self, paused: Pausable<Lint>) -> Vec<Finding> {
        let shown = self.source.shown();
        paused
            .woken()
            .map_or_default(|lint| lint.found().into_iter().map(|found| found.in_file(Lint::RULE, shown)).collect())
    }

    fn facts<Lint: Graph<'run>>(paused: Pausable<Lint>) -> Vec<Lint::Fact> {
        paused.woken().map_or_default(Graph::facts)
    }
}

impl Facts {
    pub(crate) fn joined(self, later: Facts) -> Facts {
        Facts {
            surface: self.surface.into_iter().chain(later.surface).collect(),
            api: self.api.into_iter().chain(later.api).collect(),
            test_only: self.test_only.into_iter().chain(later.test_only).collect(),
            modules: self.modules.into_iter().chain(later.modules).collect(),
        }
    }

    pub(crate) fn reduced(self, packages: &[Arc<Package>]) -> Vec<Finding> {
        let surface = PublicSurface::reduce(self.surface, packages);
        let api = PublicApiTests::reduce(self.api, packages);
        let test_only = NoTestOnlyApi::reduce(self.test_only, packages);
        let modules = ModuleCycles::reduce(self.modules, packages);
        surface.into_iter().chain(api).chain(test_only).chain(modules).collect()
    }
}
