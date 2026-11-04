mod behaviour;
mod coloured;
mod structure;

use equivalence::{Equivalence, Erased, Groups, Laws};
use lint_core::{Headline, Report, Violation};

use crate::algebra_laws::attached::Attached;
use crate::edge::Edge;
use crate::graph_lint::GraphLint;
use crate::system::TypeSystem;
use behaviour::Behaviour;
use coloured::Coloured;
use structure::{Structure, Tally};

#[derive(Debug)]
pub struct ParallelStructures;

impl ParallelStructures {
    const EQUIVALENCE: Equivalence = Equivalence::erasing(&[Erased::Names, Erased::TraitLabels], Laws::WhereDeclared);
}

impl GraphLint for ParallelStructures {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let attached = Attached::declared(system);
        let behaviour = Behaviour::read(system, &attached, Self::EQUIVALENCE);
        let colours = behaviour.refinement().coloured();
        let structures: Vec<Structure<'_, 'file>> =
            Groups::grouped(colours.iter().map(|(node, colour)| Coloured::new(*node, *colour)))
                .listed()
                .iter()
                .filter_map(|group| Structure::renamed(group, &behaviour, &colours))
                .collect();
        Report::new(
            Headline::new(
                "report only, parallel structures: colour refinement over the domain types on their moves, each move labelled by its role (written, operation, port, lift, store) with names and trait labels erased, so a trait impl is a written move, the black box it reaches outside the domain, and the colours of the type it reaches and of the other inputs it takes with; settings: types of one colour are one structure written again when their moves map onto each other under one consistent renaming of the types they reach, checked one hop deep, and where both moves declare laws the laws match; groups of two or more types making two or more moves",
            ),
            Tally::counted(&colours, &structures)
                .placed(system.edges().iter().map(Edge::site).min())
                .into_iter()
                .chain(structures.iter().filter_map(Option::<Violation<'file>>::from))
                .collect(),
        )
    }
}
