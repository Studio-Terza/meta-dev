use algebra::{Monoid, Semigroup};
use syntax::{Attribute, ItemImpl, ItemMod};

use graph::{Ground, Reach, Region, Within, Writing, Zone};
use lint_core::{Gate, SourceFile, Walk};

use super::places::PlaceId;
use crate::namespace::ScopeKind;

#[derive(Clone, Copy, Debug)]
pub(super) struct Regions;

impl Regions {
    pub(super) fn of_file(source: &SourceFile, reach: Reach) -> Region {
        let head = source.head();
        let ground = Gate::PORTS
            .admits(head)
            .chosen(Ground::Ports, Gate::STORE.admits(head).chosen(Ground::Store, Ground::Open));
        Region::new(reach, Self::zone_of(source), Within::Own).combine(Region::from(ground))
    }

    pub(super) fn of_module(item: &ItemMod) -> Region {
        let ground =
            Gate::PORTS.names(item).chosen(Ground::Ports, Gate::STORE.names(item).chosen(Ground::Store, Ground::Open));
        let zone = Gate::BOUNDARY.names(item).chosen(Zone::Boundary, Zone::Open);
        Monoid::concat([Self::tested(&item.attrs), Region::from(zone), Region::from(ground)])
    }

    fn tested(attributes: &[Attribute]) -> Region {
        match Walk::outside_tests(attributes) {
            Walk::Skipped => Region::from(Reach::Test),
            Walk::Entered => Region::default(),
        }
    }

    fn zone_of(source: &SourceFile) -> Zone {
        Gate::BOUNDARY.admits(source.head()).chosen(Zone::Boundary, Zone::Open)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MacroItems {
    Transcribed,
    Expanded,
}

impl From<MacroItems> for Region {
    fn from(items: MacroItems) -> Region {
        match items {
            MacroItems::Transcribed => Region::from(Reach::Transcribed),
            MacroItems::Expanded => Region::from(Writing::Expanded),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spot {
    Declared,
    Body,
}

impl From<ScopeKind> for Spot {
    fn from(kind: ScopeKind) -> Spot {
        match kind {
            ScopeKind::Module => Spot::Declared,
            ScopeKind::Block => Spot::Body,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Context {
    region: Region,
    place: Option<PlaceId>,
    spot: Spot,
}

impl Context {
    pub(super) fn outermost(region: Region) -> Context {
        Context { region, place: None, spot: Spot::Declared }
    }

    pub(super) fn within_item(&self, attributes: &[Attribute]) -> Context {
        Context { region: self.region.combine(Regions::tested(attributes)), ..self.clone() }
    }

    pub(super) fn within_module(&self, item: &ItemMod) -> Context {
        Context { region: self.region.combine(Regions::of_module(item)), ..self.clone() }
    }

    pub(super) fn of_impl(&self, block: &ItemImpl, place: PlaceId) -> Context {
        Context { region: self.region.combine(Region::from(Self::within(block))), place: Some(place), spot: self.spot }
    }

    pub(super) fn within_macro(&self, items: MacroItems) -> Context {
        Context { region: self.region.combine(Region::from(items)), ..self.clone() }
    }

    pub(super) fn owned(&self, place: PlaceId) -> Context {
        Context { place: Some(place), region: self.region, spot: self.spot }
    }

    pub(super) fn entered(&self, kind: ScopeKind) -> Context {
        Context { spot: Spot::from(kind), ..self.clone() }
    }

    pub(super) fn in_body(&self) -> Option<Context> {
        Some(self.clone()).filter(|context| context.spot == Spot::Body)
    }

    pub(super) fn region(&self) -> Region {
        self.region
    }

    pub(super) fn place(&self) -> Option<PlaceId> {
        self.place.clone()
    }

    fn within(block: &ItemImpl) -> Within {
        match block.trait_ {
            Some(_) => Within::TraitImpl,
            None => Within::Own,
        }
    }
}
