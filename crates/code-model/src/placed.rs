use lint_layout::Name;

use crate::site::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed<'model, Entity> {
    site: Site<'model>,
    krate: &'model Name,
    entity: Entity,
}

impl<'model, Entity: Copy> Placed<'model, Entity> {
    #[must_use]
    pub fn new(site: Site<'model>, krate: &'model Name, entity: Entity) -> Placed<'model, Entity> {
        Placed { site, krate, entity }
    }

    #[must_use]
    pub fn site(self) -> Site<'model> {
        self.site
    }

    #[must_use]
    pub fn krate(self) -> &'model Name {
        self.krate
    }

    #[must_use]
    pub fn entity(self) -> Entity {
        self.entity
    }
}
