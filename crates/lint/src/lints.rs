mod bool_methods;
mod expression_flow;
mod manual_iteration;
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

use timer_ports::Timer;

use crate::finding::Finding;
use crate::graph::Graph;
use crate::source::Source;
use crate::spent::{Spent, Stage};
use crate::style::Style;
use crate::workspace::Package;
use bool_methods::BoolMethods;
use expression_flow::ExpressionFlow;
use manual_iteration::ManualIteration;
use named_lifetimes::NamedLifetimes;
use no_comments::NoComments;
use no_dyn::NoDyn;
use no_free_fns::NoFreeFns;
use no_test_only_api::NoTestOnlyApi;
pub(crate) use process_starters::ProcessStarters;
use public_api_tests::{ApiFact, PublicApiTests};
use public_surface::{PublicSurface, SurfaceFact};
use type_shape::TypeShape;
use vec_out_params::VecOutParams;

#[derive(Debug, Default)]
pub(crate) struct Facts {
    surface: Vec<SurfaceFact>,
    api: Vec<ApiFact>,
    test_only: Vec<Finding>,
}

#[derive(Debug, Default)]
pub(crate) struct Linted {
    pub found: Vec<Finding>,
    pub facts: Facts,
}

#[derive(Debug)]
pub(crate) struct Lints<'run, Clock> {
    source: &'run Source,
    file: &'run syn::File,
    timer: &'run Clock,
}

impl<'run, Clock: Timer> Lints<'run, Clock> {
    pub(crate) fn linted(source: &'run Source, file: &'run syn::File, timer: &'run Clock, spent: &mut Spent) -> Linted {
        let lints = Lints { source, file, timer };
        let found = [
            lints.styled::<NoComments>(spent),
            lints.styled::<NoFreeFns>(spent),
            lints.styled::<VecOutParams>(spent),
            lints.styled::<NamedLifetimes>(spent),
            lints.styled::<TypeShape>(spent),
            lints.styled::<ExpressionFlow>(spent),
            lints.styled::<ManualIteration>(spent),
            lints.styled::<ProcessStarters>(spent),
            lints.styled::<BoolMethods>(spent),
            lints.styled::<NoDyn>(spent),
        ];
        let facts = Facts {
            surface: lints.graphed::<PublicSurface>(spent),
            api: lints.graphed::<PublicApiTests>(spent),
            test_only: lints.graphed::<NoTestOnlyApi>(spent),
        };
        Linted { found: found.into_iter().flatten().collect(), facts }
    }

    fn styled<Lint: Style<'run>>(&self, spent: &mut Spent) -> Vec<Finding> {
        spent.timed(self.timer, Stage::Visit(Lint::RULE), || {
            Lint::READS.admits(self.source).map_or_default(|()| {
                let mut lint = Lint::new(self.source);
                lint.visit_file(self.file);
                let shown = self.source.shown();
                lint.found().into_iter().map(|found| found.in_file(Lint::RULE, shown)).collect()
            })
        })
    }

    fn graphed<Lint: Graph<'run>>(&self, spent: &mut Spent) -> Vec<Lint::Fact> {
        spent.timed(self.timer, Stage::Visit(Lint::RULE), || {
            let mut lint = Lint::new(self.source);
            lint.visit_file(self.file);
            lint.facts()
        })
    }
}

impl Facts {
    pub(crate) fn joined(self, later: Facts) -> Facts {
        Facts {
            surface: self.surface.into_iter().chain(later.surface).collect(),
            api: self.api.into_iter().chain(later.api).collect(),
            test_only: self.test_only.into_iter().chain(later.test_only).collect(),
        }
    }

    pub(crate) fn reduced(self, packages: &[Arc<Package>]) -> Vec<Finding> {
        let surface = PublicSurface::reduce(self.surface, packages);
        let api = PublicApiTests::reduce(self.api, packages);
        let test_only = NoTestOnlyApi::reduce(self.test_only, packages);
        surface.into_iter().chain(api).chain(test_only).collect()
    }
}
