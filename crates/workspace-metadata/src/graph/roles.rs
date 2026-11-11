mod breaching;
mod expansion;
mod owned;
mod reformatted;
mod unleafed;
mod unported;
mod unroled;

use super::Graph;
use super::unit::Unit;
pub use breaching::Breaching;
pub use expansion::Expansion;
use owned::Owned;
pub use reformatted::Reformatted;
pub use unleafed::Unleafed;
pub use unported::Unported;
pub use unroled::Unroled;

#[derive(Debug)]
pub struct RoleBreaches<'meta> {
    unroled: Vec<Unroled<'meta>>,
    unleafed: Vec<Unleafed<'meta>>,
    reformatted: Vec<Reformatted<'meta>>,
    unported: Vec<Unported<'meta>>,
}

impl<'meta> RoleBreaches<'meta> {
    pub(super) fn found(graph: &Graph<'meta>, units: &[Unit<'meta>]) -> RoleBreaches<'meta> {
        let owned = Owned::held(graph, units);
        RoleBreaches {
            unroled: Unroled::found(graph),
            unleafed: Unleafed::found(graph, &owned),
            reformatted: Reformatted::found(graph),
            unported: Unported::found(graph, &owned),
        }
    }

    #[must_use]
    pub fn unroled(&self) -> &[Unroled<'meta>] {
        &self.unroled
    }

    #[must_use]
    pub fn unleafed(&self) -> &[Unleafed<'meta>] {
        &self.unleafed
    }

    #[must_use]
    pub fn reformatted(&self) -> &[Reformatted<'meta>] {
        &self.reformatted
    }

    #[must_use]
    pub fn unported(&self) -> &[Unported<'meta>] {
        &self.unported
    }
}
