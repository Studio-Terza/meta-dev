use syn::spanned::Spanned;
use syn::{Attribute, ItemImpl, ItemMod};

use code_model::{Reach, Region, TypeRef, Within, Zone};
use lint_core::{Gate, SourceFile, Walk};
use lint_layout::LineNumber;

use crate::namespace::UsePath;

use super::held::{Link, SlotRow};
use super::signature::Written;
use super::{Lowering, Mode};

#[derive(Clone, Copy, Debug)]
pub(super) struct Regions;

impl Regions {
    pub(super) fn of_file(source: &SourceFile) -> Region {
        Region::new(Self::reach_of(source), Self::zone_of(source), Within::Own)
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

    fn reach_of(source: &SourceFile) -> Reach {
        match source.shipped() {
            Some(_) => Reach::Shipped,
            None => Reach::Test,
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
    interface: Option<Written>,
}

impl Context {
    pub(super) fn outermost(region: Region) -> Context {
        Context { region, subject: None, interface: None }
    }

    pub(super) fn within_module(&self, item: &ItemMod) -> Context {
        Context {
            region: Regions::of_module(self.region, item),
            subject: self.subject.clone(),
            interface: self.interface.clone(),
        }
    }

    pub(super) fn of_impl(&self, block: &ItemImpl, subject: TypeRef<Written>) -> Context {
        let interface =
            block.trait_.as_ref().map(|(_, path, _)| Written::path(UsePath::from(path), LineNumber::from(path.span())));
        Context { region: self.region.inside(Self::within(block)), subject: Some(subject), interface }
    }

    pub(super) fn of_trait(&self) -> Context {
        Context { region: self.region, subject: None, interface: None }
    }

    pub(super) fn reached(&self, reach: Reach) -> Context {
        Context { region: self.region.reached(reach), subject: self.subject.clone(), interface: self.interface.clone() }
    }

    pub(super) fn region(&self) -> Region {
        self.region
    }

    pub(super) fn subject(&self) -> Option<TypeRef<Written>> {
        self.subject.clone()
    }

    pub(super) fn slot(&self) -> Link {
        Link::from(self.subject.clone().map(|subject| SlotRow::new(subject, self.interface.clone())))
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

    pub(super) fn expansion_visited(&mut self, visited: impl FnOnce(&mut Self)) {
        let context = self.context.reached(Reach::MacroBody);
        let mode = std::mem::replace(&mut self.mode, Mode::Expansion);
        self.contextual(context, visited);
        self.mode = mode;
    }
}
