mod disguised;
mod shape;

use lint_core::{Headline, Report, Violation};

use crate::GraphLint;
use crate::system::TypeSystem;
use disguised::Disguised;
use shape::{Shaped, Shapes};

#[derive(Debug)]
pub struct UndeclaredAlgebra;

impl GraphLint for UndeclaredAlgebra {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let shaped: Vec<Shaped<'_, 'file>> = system.edges().iter().filter_map(Shaped::of_edge).collect();
        let disguised = Disguised::found(system.model());
        let tallied = Shapes::tallied(&shaped, &disguised);
        Report::new(
            Headline::new(
                "report only, undeclared algebra (transition-graph D12): in core modules, an edge from a type back to the same type is refused unless it is the combine of a declared algebra::Semigroup; a unary T -> T is a missing refinement type, a builder (T, X) -> T is always refused, a binary (T, T) -> T must be a Semigroup's combine, and &mut, &mut self included, belongs in a store operation; std::ops::Add with Output = Self needs algebra::Semigroup, and Extend<Self> or FromIterator<Self> for Self needs algebra::Monoid; signatures a foreign trait fixes are counted apart",
            ),
            shaped
                .iter()
                .filter_map(Option::<Violation<'file>>::from)
                .chain(disguised.iter().map(Violation::from))
                .chain(Option::<Violation<'file>>::from(tallied))
                .collect(),
        )
    }
}
