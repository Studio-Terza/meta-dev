mod behaviour;
mod coloured;
mod fillers;
mod structure;

use equivalence::{Equivalence, Erased, Laws};
use lint_core::{Headline, Report, Violation};

use crate::attached::Attached;
use crate::edge::Edge;
use crate::graph_lint::GraphLint;
use crate::system::TypeSystem;
use behaviour::Behaviour;
use coloured::Coloured;
use fillers::Fillers;
use structure::{Structure, Tally};

#[derive(Debug)]
pub struct ParallelStructures;

impl ParallelStructures {
    const EQUIVALENCE: Equivalence =
        Equivalence::erasing(&[Erased::Names, Erased::Literals]).comparing(Laws::WhereDeclared);
}

impl GraphLint for ParallelStructures {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let attached = Attached::declared(system);
        let behaviour = Behaviour::read(system, &attached, Self::EQUIVALENCE);
        let fillers = Fillers::read(system.files(), Self::EQUIVALENCE);
        let coloured = Coloured::refined(behaviour.refinement());
        let structures: Vec<Structure<'_, 'file>> = behaviour
            .refinement()
            .clustered(coloured.colours(), |first, other| fillers.alike(first, other))
            .iter()
            .filter_map(|cluster| Structure::of_cluster(cluster, &behaviour))
            .collect();
        Report::new(
            Headline::new(
                "report only, parallel structures: colour refinement over the domain types on their moves beyond any trait, each move labelled by its role (written, operation, port, lift, store) with names erased, the black box it reaches outside the domain, and the colours of the type it reaches and of the other inputs it takes with; moves of the same shape are counted each; trait impls are the shared interface of their fillers, so they are no evidence and are set aside; settings: within one colour, types are one structure when every move maps onto a move of the same label under one consistent renaming of the types they reach, checked one hop deep against the first type of each structure, where both moves declare laws the laws match, and where two types fill one interface their impls of it have the same bodies with names and literals erased; groups of two or more types making two or more moves beyond any trait; each type is located at its declaration",
            ),
            Tally::counted(&coloured, &structures)
                .placed(system.edges().iter().map(Edge::site).min())
                .into_iter()
                .chain(structures.iter().filter_map(Option::<Violation<'file>>::from))
                .collect(),
        )
    }
}
