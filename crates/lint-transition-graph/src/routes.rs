mod leg;
mod route;
mod stops;
mod walk;

use lint_core::{Headline, Report, Violation};

use crate::edge::Edge;
use crate::graph_lint::GraphLint;
use crate::system::TypeSystem;
use equivalence::Groups;
use route::{Route, Tally};
use stops::Stops;
use walk::Walk;

#[derive(Debug)]
pub struct RepeatedRoutes;

impl GraphLint for RepeatedRoutes {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let stops = Stops::indexed(system.edges());
        let walks: Vec<Walk<'_, '_, 'file>> = system
            .edges()
            .iter()
            .filter(|edge| edge.form().written().is_some())
            .map(|edge| stops.walked(edge))
            .collect();
        let routes: Vec<Route<'_, '_, '_, 'file>> = Groups::grouped(walks.iter().flat_map(Walk::legs))
            .ranked(|stops| stops.len())
            .maximal()
            .listed()
            .into_iter()
            .filter_map(|group| Route::of_group(group, &stops))
            .collect();
        let tally = Tally::counted(&walks, &routes);
        Report::new(
            Headline::new(
                "report only, repeated routes (path thinness): each written body's calls, in the order they run, resolved by name to the one workspace edge of that name and read as that edge's hop from its domain inputs to its domain output, names and trait labels erased; settings: a route is a run of three or more hops, at least two of them distinct, walked in two or more different bodies, and a route repeated inside one body is not counted; calls to std, to names several edges share, and to edges without a domain end are left out; a route whose first inputs and last output are already joined by one written edge says so, the commuting triangle of the old report",
            ),
            tally
                .placed(system.edges().iter().map(Edge::site).min())
                .into_iter()
                .chain(routes.iter().filter_map(Option::<Violation<'file>>::from))
                .collect(),
        )
    }
}
