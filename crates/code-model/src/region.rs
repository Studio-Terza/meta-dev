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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    Open,
    Ports,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    reach: Reach,
    zone: Zone,
    within: Within,
    ground: Ground,
}

impl Region {
    pub const SHIPPED: Region =
        Region { reach: Reach::Shipped, zone: Zone::Open, within: Within::Own, ground: Ground::Open };

    #[must_use]
    pub fn new(reach: Reach, zone: Zone, within: Within) -> Region {
        Region { reach, zone, within, ground: Ground::Open }
    }

    #[must_use]
    pub fn ground(self) -> Ground {
        self.ground
    }

    #[must_use]
    pub fn grounded(self, ground: Ground) -> Region {
        Region { ground: self.ground.max(ground), ..self }
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
    pub fn reached(self, reach: Reach) -> Region {
        Region { reach: self.reach.max(reach), ..self }
    }

    #[must_use]
    pub fn zoned(self, zone: Zone) -> Region {
        Region { zone: self.zone.max(zone), ..self }
    }

    #[must_use]
    pub fn inside(self, within: Within) -> Region {
        Region { within: self.within.max(within), ..self }
    }

    #[must_use]
    pub fn joined(self, other: Region) -> Region {
        self.reached(other.reach).zoned(other.zone).inside(other.within).grounded(other.ground)
    }

    #[must_use]
    pub fn ungrounded(self) -> Region {
        Region { ground: Ground::Open, ..self }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    region: Region,
    subject: Option<TypeRef>,
}

impl Seat {
    #[must_use]
    pub fn new(region: Region, subject: Option<TypeRef>) -> Seat {
        Seat { region, subject }
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
