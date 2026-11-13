use code_model::{Ground as Grounded, Region, Zone};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    Core,
    Ports,
    Boundary,
    Store,
}

impl Ground {
    fn gated(ground: Grounded) -> Option<Ground> {
        match ground {
            Grounded::Ports => Some(Ground::Ports),
            Grounded::Store => Some(Ground::Store),
            Grounded::Open => None,
        }
    }

    fn zoned(zone: Zone) -> Ground {
        match zone {
            Zone::Boundary => Ground::Boundary,
            Zone::Open => Ground::Core,
        }
    }
}

impl From<Region> for Ground {
    fn from(region: Region) -> Ground {
        Ground::gated(region.ground()).unwrap_or_else(|| Ground::zoned(region.zone()))
    }
}
