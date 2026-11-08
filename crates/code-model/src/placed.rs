use crate::site::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed<'model, Entity> {
    site: Site<'model>,
    entity: Entity,
}

impl<'model, Entity: Copy> Placed<'model, Entity> {
    #[must_use]
    pub fn new(site: Site<'model>, entity: Entity) -> Placed<'model, Entity> {
        Placed { site, entity }
    }

    #[must_use]
    pub fn site(self) -> Site<'model> {
        self.site
    }

    #[must_use]
    pub fn entity(self) -> Entity {
        self.entity
    }
}
