use port_core::{Calls, Op, Run};

use crate::{Bound, Complexity, Scale};

pub trait Workload {
    type Fixture;
    type Done: std::fmt::Display;
    type Failure: std::fmt::Debug + 'static;
    type Scaled: Op;
    type Listed: Op;

    const CLASS: Complexity;

    fn fixture(scale: Scale) -> Self::Fixture;

    fn inputs(scale: Scale) -> Calls;

    fn run(run: &Run, fixture: Self::Fixture) -> Result<Self::Done, Self::Failure>;

    #[must_use]
    fn bounds(scale: Scale) -> Vec<Bound> {
        vec![Bound::at_most::<Self::Scaled>(Self::inputs(scale)), Bound::at_most::<Self::Listed>(Calls::new(1))]
    }
}
