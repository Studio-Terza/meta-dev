use algebra::{Monoid, Semigroup};
use lint_layout::Name;
use newtype::Arbitrary;

use crate::boundary::ItemId;
use crate::resolved::Resolved;
use crate::type_ref::TypeRef;

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    #[default]
    Shipped,
    Transcribed,
    Test,
}

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Zone {
    #[default]
    Open,
    Boundary,
}

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Within {
    #[default]
    Own,
    TraitImpl,
}

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    #[default]
    Open,
    Store,
    Ports,
}

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Writing {
    #[default]
    Written,
    Expanded,
}

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Region {
    reach: Reach,
    zone: Zone,
    within: Within,
    ground: Ground,
    writing: Writing,
}

impl Region {
    pub const SHIPPED: Region = Region {
        reach: Reach::Shipped,
        zone: Zone::Open,
        within: Within::Own,
        ground: Ground::Open,
        writing: Writing::Written,
    };

    #[must_use]
    pub fn new(reach: Reach, zone: Zone, within: Within) -> Region {
        Region { reach, zone, within, ground: Ground::Open, writing: Writing::Written }
    }

    #[must_use]
    pub fn writing(self) -> Writing {
        self.writing
    }

    #[must_use]
    pub fn ground(self) -> Ground {
        self.ground
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
}

impl Semigroup for Region {
    fn combine(self, other: Region) -> Region {
        Region {
            reach: self.reach.max(other.reach),
            zone: self.zone.max(other.zone),
            within: self.within.max(other.within),
            ground: self.ground.max(other.ground),
            writing: self.writing.max(other.writing),
        }
    }
}

impl Monoid for Region {}

impl From<Reach> for Region {
    fn from(reach: Reach) -> Region {
        Region { reach, ..Region::SHIPPED }
    }
}

impl From<Zone> for Region {
    fn from(zone: Zone) -> Region {
        Region { zone, ..Region::SHIPPED }
    }
}

impl From<Within> for Region {
    fn from(within: Within) -> Region {
        Region { within, ..Region::SHIPPED }
    }
}

impl From<Ground> for Region {
    fn from(ground: Ground) -> Region {
        Region { ground, ..Region::SHIPPED }
    }
}

impl From<Writing> for Region {
    fn from(writing: Writing) -> Region {
        Region { writing, ..Region::SHIPPED }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    region: Region,
    subject: Option<TypeRef>,
    owner: Option<ItemId>,
    implemented: Option<Name>,
    traited: Option<Resolved>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unowned {
    region: Region,
    subject: Option<TypeRef>,
}

impl Unowned {
    #[must_use]
    pub fn new(region: Region, subject: Option<TypeRef>) -> Unowned {
        Unowned { region, subject }
    }

    #[must_use]
    pub fn owned(self, owner: Option<ItemId>, implemented: Option<Name>, traited: Option<Resolved>) -> Seat {
        Seat { region: self.region, subject: self.subject, owner, implemented, traited }
    }
}

impl Seat {
    #[must_use]
    pub fn implemented_trait(&self) -> Option<&Resolved> {
        self.traited.as_ref()
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&Name> {
        self.implemented.as_ref()
    }

    #[must_use]
    pub fn subject(&self) -> Option<&TypeRef> {
        self.subject.as_ref()
    }

    #[must_use]
    pub fn owner(&self) -> Option<ItemId> {
        self.owner
    }
}

impl AsRef<Region> for Seat {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}
