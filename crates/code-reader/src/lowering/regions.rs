use syn::{Attribute, ItemImpl, ItemMod};

use code_model::{Reach, Region, TypeRef, Within, Zone};
use lint_core::{Gate, SourceFile, Walk};

use crate::planting::ReadFile;

use super::Lowering;
use super::signature::Written;

#[derive(Clone, Copy, Debug)]
pub(super) struct Regions;

impl Regions {
    pub(super) fn of_file(read: ReadFile<'_>) -> Region {
        Region::new(read.reach(), Self::zone_of(read.source()), Within::Own)
    }

    pub(super) fn of_module(region: Region, item: &ItemMod) -> Region {
        match Gate::BOUNDARY.names(item) {
            Some(_) => Self::tested(region, &item.attrs).zoned(Zone::Boundary),
            None => Self::tested(region, &item.attrs),
        }
    }

    pub(super) fn joined(slot: Region, file: Region) -> Region {
        slot.reached(file.reach()).zoned(file.zone())
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
    subject: Option<TypeRef<Written>>,
}

impl Context {
    pub(super) fn outermost(region: Region) -> Context {
        Context { region, subject: None }
    }

    pub(super) fn within_module(&self, item: &ItemMod) -> Context {
        Context { region: Regions::of_module(self.region, item), subject: self.subject.clone() }
    }

    pub(super) fn of_impl(&self, block: &ItemImpl, subject: TypeRef<Written>) -> Context {
        Context { region: self.region.inside(Self::within(block)), subject: Some(subject) }
    }

    pub(super) fn of_trait(&self) -> Context {
        Context { region: self.region, subject: None }
    }

    pub(super) fn region(&self) -> Region {
        self.region
    }

    pub(super) fn subject(&self) -> Option<TypeRef<Written>> {
        self.subject.clone()
    }

    fn within(block: &ItemImpl) -> Within {
        match block.trait_ {
            Some(_) => Within::TraitImpl,
            None => Within::Own,
        }
    }
}

impl Lowering<'_, '_> {
    pub(super) fn contextual(&mut self, context: Context, visited: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.context, context);
        visited(self);
        self.context = outer;
    }
}
