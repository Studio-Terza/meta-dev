use syn::{Attribute, ItemImpl, ItemMod};

use graph::{Ground, Reach, Region, Within, Zone};
use lint_core::{Gate, SourceFile, Walk};

use super::places::PlaceId;

#[derive(Clone, Copy, Debug)]
pub(super) struct Regions;

impl Regions {
    pub(super) fn of_file(source: &SourceFile, reach: Reach) -> Region {
        let head = source.head();
        let ported = Gate::PORTS.admits(head).map(|_| Ground::Ports);
        let ground = ported.or_else(|| Gate::STORE.admits(head).map(|_| Ground::Store)).unwrap_or(Ground::Open);
        Region::new(reach, Self::zone_of(source), Within::Own).grounded(ground)
    }

    pub(super) fn of_module(region: Region, item: &ItemMod) -> Region {
        let ported = Gate::PORTS.names(item).map(|_| Ground::Ports);
        let ground = ported.or_else(|| Gate::STORE.names(item).map(|_| Ground::Store)).unwrap_or(Ground::Open);
        let zone = Gate::BOUNDARY.names(item).map_or(Zone::Open, |_| Zone::Boundary);
        Self::tested(region, &item.attrs).zoned(zone).grounded(ground)
    }

    fn tested(region: Region, attributes: &[Attribute]) -> Region {
        match Walk::outside_tests(attributes) {
            Walk::Skipped => region.reached(Reach::Test),
            Walk::Entered => region,
        }
    }

    fn zone_of(source: &SourceFile) -> Zone {
        match Gate::BOUNDARY.admits(source.head()) {
            Some(_) => Zone::Boundary,
            None => Zone::Open,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Context {
    region: Region,
    place: Option<PlaceId>,
}

impl Context {
    pub(super) fn outermost(region: Region) -> Context {
        Context { region, place: None }
    }

    pub(super) fn within_item(&self, attributes: &[Attribute]) -> Context {
        Context { region: Regions::tested(self.region, attributes), place: self.place }
    }

    pub(super) fn within_module(&self, item: &ItemMod) -> Context {
        Context { region: Regions::of_module(self.region, item), place: self.place }
    }

    pub(super) fn of_impl(&self, block: &ItemImpl, place: PlaceId) -> Context {
        Context { region: self.region.inside(Self::within(block)), place: Some(place) }
    }

    pub(super) fn transcribed(&self) -> Context {
        Context { region: self.region.reached(Reach::Transcribed), place: self.place }
    }

    pub(super) fn owned(&self, place: PlaceId) -> Context {
        Context { region: self.region, place: Some(place) }
    }

    pub(super) fn region(&self) -> Region {
        self.region
    }

    pub(super) fn place(&self) -> Option<PlaceId> {
        self.place
    }

    fn within(block: &ItemImpl) -> Within {
        match block.trait_ {
            Some(_) => Within::TraitImpl,
            None => Within::Own,
        }
    }
}
