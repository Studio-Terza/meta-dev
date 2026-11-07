use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    Shipped,
    Test,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Zone {
    Open,
    Boundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Within {
    Own,
    TraitImpl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    reach: Reach,
    zone: Zone,
    within: Within,
}

impl Region {
    pub const SHIPPED: Self = Self { reach: Reach::Shipped, zone: Zone::Open, within: Within::Own };

    #[must_use]
    pub fn new(reach: Reach, zone: Zone, within: Within) -> Self {
        Self { reach, zone, within }
    }

    #[must_use]
    pub fn reach(self) -> Reach {
        self.reach
    }

    #[must_use]
    pub fn zone(self) -> Zone {
        self.zone
    }

    #[must_use]
    pub fn within(self) -> Within {
        self.within
    }

    #[must_use]
    pub fn reached(self, reach: Reach) -> Self {
        Self { reach: self.reach.max(reach), zone: self.zone, within: self.within }
    }

    #[must_use]
    pub fn zoned(self, zone: Zone) -> Self {
        Self { reach: self.reach, zone: self.zone.max(zone), within: self.within }
    }

    #[must_use]
    pub fn inside(self, within: Within) -> Self {
        Self { reach: self.reach, zone: self.zone, within: self.within.max(within) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    region: Region,
    subject: Option<TypeRef>,
}

impl Seat {
    #[must_use]
    pub fn new(region: Region, subject: Option<TypeRef>) -> Self {
        Self { region, subject }
    }

    #[must_use]
    pub fn subject(&self) -> Option<&TypeRef> {
        self.subject.as_ref()
    }
}

impl AsRef<Region> for Seat {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}
