use std::path::Path;

use code_model::{Held, Holding, ItemId, Model, Placed, Projection, Reach, Region, Resolved, Sited, TypeRef};
use lint_layout::Name;

use crate::legacy::{Exempting, Legacy};

#[derive(Debug)]
pub struct AliasTarget<'model> {
    item: ItemId,
    path: &'model Path,
    target: &'model TypeRef,
}

impl<'model> AliasTarget<'model> {
    fn declared(placed: Placed<'model, &'model Holding>) -> Option<AliasTarget<'model>> {
        let holding = placed.entity();
        let alias = [holding.held()].into_iter().find(|held| *held == Held::Alias);
        alias.and(holding.declares()).map(|item| AliasTarget {
            item,
            path: placed.site().path(),
            target: holding.as_ref(),
        })
    }

    pub fn same_file(&self, file: &Path) -> Legacy {
        match [self.path].into_iter().find(|path| *path == file) {
            Some(_) => Legacy::Eligible,
            None => Legacy::Ineligible,
        }
    }

    pub fn target(&self) -> &'model TypeRef {
        self.target
    }
}

#[derive(Debug)]
struct ProjectionTarget<'model> {
    subject: &'model Resolved,
    interface: &'model Resolved,
    item: &'model Name,
    target: &'model TypeRef,
}

impl<'model> ProjectionTarget<'model> {
    fn slotted(holding: &'model Holding) -> Option<ProjectionTarget<'model>> {
        let associated = [holding.held()].into_iter().find(|held| *held == Held::AssociatedType);
        let slot = associated.and(holding.slot());
        let subject = slot.and_then(|slot| AsRef::<TypeRef>::as_ref(slot).named()).map(AsRef::<Resolved>::as_ref);
        let interface = slot.and_then(|slot| slot.interface()).map(AsRef::<Resolved>::as_ref);
        subject.zip(interface).map(|(subject, interface)| ProjectionTarget {
            subject,
            interface,
            item: holding.as_ref(),
            target: holding.as_ref(),
        })
    }

    fn matching(&self, subject: &Resolved, interface: &Resolved, item: &Name) -> Option<&'model TypeRef> {
        let subject = [self.subject].into_iter().find(|target| *target == subject);
        let interface = [self.interface].into_iter().find(|target| *target == interface);
        let item = [self.item].into_iter().find(|named| *named == item);
        subject.and(interface).and(item).map(|_| self.target)
    }
}

#[derive(Debug)]
pub struct Lookups<'model> {
    aliases: Vec<AliasTarget<'model>>,
    projections: Vec<ProjectionTarget<'model>>,
    newtypes: Vec<Exempting<ItemId>>,
}

impl<'model> Lookups<'model> {
    pub fn read(model: &'model Model) -> Lookups<'model> {
        Lookups {
            aliases: model.holdings().filter_map(AliasTarget::declared).collect(),
            projections: model.holdings().filter_map(|placed| ProjectionTarget::slotted(placed.entity())).collect(),
            newtypes: model.holdings().filter_map(|placed| Lookups::newtype_of(placed.entity())).collect(),
        }
    }

    pub fn alias(&self, resolved: &Resolved) -> Option<&AliasTarget<'model>> {
        resolved.crate_item().and_then(|item| self.aliases.iter().find(|alias| alias.item == item))
    }

    pub fn projected(&self, projection: &Projection<Sited>) -> Option<&'model TypeRef> {
        let subject = projection.subject().named().map(AsRef::<Resolved>::as_ref);
        let interface = projection.interface().map(AsRef::<Resolved>::as_ref);
        let item: &Name = projection.as_ref();
        subject.zip(interface).and_then(|(subject, interface)| {
            self.projections.iter().find_map(|target| target.matching(subject, interface, item))
        })
    }

    pub fn newtype(&self, item: Option<ItemId>, reach: Reach) -> Option<ItemId> {
        item.and_then(|item| {
            self.newtypes
                .iter()
                .filter_map(|newtype| newtype.covering(reach))
                .find(|newtype| **newtype == item)
                .copied()
        })
    }

    fn newtype_of(holding: &Holding) -> Option<Exempting<ItemId>> {
        let newtype = [holding.held()].into_iter().find(|held| *held == Held::NewtypeField);
        let region: &Region = holding.as_ref();
        newtype.and(holding.declares()).map(|item| Exempting::new(item, region.reach()))
    }
}
