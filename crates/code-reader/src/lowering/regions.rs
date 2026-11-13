use syn::{Attribute, ItemImpl, ItemMod};

use code_model::{Ground, Reach, Region, Within, Zone};
use lint_core::{Gate, SourceFile, Walk};

use super::Lowering;
use super::subjects::SubjectId;

#[derive(Clone, Copy, Debug)]
pub(super) struct Regions;

impl Regions {
    pub(super) fn of_file(source: &SourceFile, reach: Reach) -> Region {
        let ground = Gate::PORTS.admits(source).map_or(Ground::Open, |_| Ground::Ports);
        Region::new(reach, Self::zone_of(source), Within::Own).grounded(ground)
    }

    pub(super) fn of_module(region: Region, item: &ItemMod) -> Region {
        let ground = Gate::PORTS.names(item).map_or(Ground::Open, |_| Ground::Ports);
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
        match Gate::BOUNDARY.admits(source) {
            Some(_) => Zone::Boundary,
            None => Zone::Open,
        }
    }
}

#[derive(Debug)]
pub(super) struct Context {
    region: Region,
    subject: Option<SubjectId>,
}

impl Context {
    pub(super) fn outermost(region: Region) -> Context {
        Context { region, subject: None }
    }

    pub(super) fn within_item(&self, attributes: &[Attribute]) -> Context {
        Context { region: Regions::tested(self.region, attributes), subject: self.subject }
    }

    pub(super) fn within_module(&self, item: &ItemMod) -> Context {
        Context { region: Regions::of_module(self.region, item), subject: self.subject }
    }

    pub(super) fn of_impl(&self, block: &ItemImpl, subject: SubjectId) -> Context {
        Context { region: self.region.inside(Self::within(block)), subject: Some(subject) }
    }

    pub(super) fn of_trait(&self) -> Context {
        Context { region: self.region, subject: None }
    }

    pub(super) fn region(&self) -> Region {
        self.region
    }

    pub(super) fn subject(&self) -> Option<SubjectId> {
        self.subject
    }

    fn within(block: &ItemImpl) -> Within {
        match block.trait_ {
            Some(_) => Within::TraitImpl,
            None => Within::Own,
        }
    }
}

impl Lowering<'_> {
    pub(super) fn contextual(&mut self, context: Context, visited: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.context, context);
        visited(self);
        self.context = outer;
    }
}
