use lint_sweep::Folding;
use syntax::{Attribute, Ident, ItemImpl, ItemMacro, ItemTrait, Macro};

use graph::{Building, FileId, ItemDeclaration, ItemId, Region, Resolved, Seat, TypeRef};
use lint_core::{Macros, Syntax};
use lint_layout::{LineNumber, Name, Position};

use crate::namespace::{Domain, Entries, PortMacro, Resolving, ScopeId, UsePath};

use super::signature::Written;
use super::{Enclosure, ImplRow, Lowering};
use lint_sweep::{Descend, Scoped, Sweep};

#[derive(Clone, Copy, Debug)]
pub(super) struct PlaceId(usize);

#[derive(Clone, Copy, Debug)]
struct SubjectId(usize);

#[derive(Debug)]
struct SubjectRow {
    scope: ScopeId,
    ty: TypeRef<Written>,
    implemented: Option<Name>,
    traited: Option<UsePath>,
}

impl SubjectRow {
    fn implemented_trait(&self, resolving: &Resolving<'_, '_, '_>) -> Option<Resolved> {
        self.traited.as_ref().map(|path| resolving.path(self.scope, path, Domain::Types))
    }
}

#[derive(Debug)]
struct OwnerRow {
    scope: ScopeId,
    name: UsePath,
    domain: Domain,
    line: LineNumber,
    region: Region,
}

impl OwnerRow {
    fn settled(&self, resolving: &Resolving<'_, '_, '_>) -> Option<ItemId> {
        resolving.path(self.scope, &self.name, self.domain).crate_item()
    }
}

#[derive(Debug)]
struct PlaceRow {
    subject: Option<SubjectId>,
    owner: Option<OwnerRow>,
}

#[derive(Debug, Default)]
pub(super) struct PlaceRows {
    subjects: Vec<SubjectRow>,
    places: Vec<PlaceRow>,
}

impl PlaceRows {
    pub(super) fn settled(&self, resolving: &Resolving<'_, '_, '_>) -> Places {
        let subjects: Vec<TypeRef> =
            self.subjects.iter().map(|row| Written::typed(&row.ty, resolving, row.scope)).collect();
        let places = self
            .places
            .iter()
            .map(|row| {
                let subject = row.subject.and_then(|id| subjects.get(id.0)).cloned();
                let declared = row.owner.as_ref().and_then(|owner| {
                    let item = owner.settled(resolving)?;
                    Some(Declared { line: owner.line, item, region: owner.region })
                });
                let owner = declared
                    .map(|found| found.item)
                    .or_else(|| subject.as_ref()?.named().map(AsRef::<Resolved>::as_ref)?.crate_item());
                let subjected = row.subject.and_then(|id| self.subjects.get(id.0));
                let implemented = subjected.and_then(|found| found.implemented.clone());
                let traited = subjected.and_then(|found| found.implemented_trait(resolving));
                Place { subject, owner, declared, implemented, traited }
            })
            .collect();
        Places(places)
    }
}

#[derive(Clone, Copy, Debug)]
struct Declared {
    line: LineNumber,
    item: ItemId,
    region: Region,
}

#[derive(Debug)]
struct Place {
    subject: Option<TypeRef>,
    owner: Option<ItemId>,
    declared: Option<Declared>,
    implemented: Option<Name>,
    traited: Option<Resolved>,
}

#[derive(Debug)]
pub(super) struct Places(Vec<Place>);

impl Places {
    pub(super) fn placed(&self, file: FileId, building: Building) -> Building {
        self.0.iter().filter_map(|place| place.declared).fold(building, |mut built, declared| {
            built.placed(file, declared.line, ItemDeclaration::new(declared.item, declared.region));
            built
        })
    }

    pub(super) fn seat(&self, region: Region, place: Option<PlaceId>) -> Seat {
        let found = place.and_then(|id| self.0.get(id.0));
        Seat::new(region, found.and_then(|seated| seated.subject.clone()))
            .owned(found.and_then(|seated| seated.owner))
            .implementing(found.and_then(|seated| seated.implemented.clone()))
            .traited(found.and_then(|seated| seated.traited.clone()))
    }
}

impl<'read> Lowering<'read> {
    fn placed(&mut self, row: PlaceRow) -> PlaceId {
        let places = &mut self.lowered.rows.places.places;
        let place = PlaceId(places.len());
        places.push(row);
        place
    }

    pub(super) fn owning(&mut self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> PlaceId {
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner = OwnerRow { scope: self.here.scope(), name: UsePath::from(ident), domain, line, region };
        self.placed(PlaceRow { subject: None, owner: Some(owner) })
    }

    pub(super) fn owned_by(&mut self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> Descend {
        let place = self.owning(ident, domain, attributes);
        self.context = self.context.owned(place);
        Descend::Into
    }

    pub(super) fn transcribed(self, item: &ItemMacro) -> Lowering<'read> {
        let statements = Macros::expanded(self.source, &item.mac);
        let outer = self.context;
        let inner = Lowering { context: self.context.transcribed(), ..self };
        let back = inner.reentered(statements.iter().filter_map(Syntax::stmt_item));
        Lowering { context: outer, ..back }
    }

    pub(super) fn expanded(self, invocation: &Macro) -> Lowering<'read> {
        let items = Macros::items_in_file(self.source, invocation);
        let mut opened = self;
        opened.lowered.openings.absorb(opened.here.scope(), Entries::of_items(&items));
        opened.reentered(items.iter())
    }

    fn reentered<'item>(self, items: impl Iterator<Item = &'item syntax::Item>) -> Lowering<'read> {
        let origin = self.origin;
        let mut scoped = Scoped::new(Folding::seeded(origin, Some(Box::new(self))));
        items.for_each(|declared| Sweep::expanded_item(declared, &mut scoped));
        *origin.opened(scoped.into_parent().concluded())
    }

    pub(super) fn trait_owning(&mut self, ident: &Ident, attributes: &[Attribute]) -> PlaceId {
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner =
            OwnerRow { scope: self.here.scope(), name: UsePath::from(ident), domain: Domain::Types, line, region };
        self.placed(PlaceRow { subject: None, owner: Some(owner) })
    }

    pub(super) fn filler_owning(&mut self, item: &ItemTrait) {
        let line = Position::from(item.ident.span()).line();
        let region = self.context.within_item(&item.attrs).region();
        let owner = PortMacro::marked(item).map(|found| OwnerRow {
            scope: self.here.scope(),
            name: UsePath::suffixed(&found.ident, PortMacro::FILLER),
            domain: Domain::Types,
            line,
            region,
        });
        owner.into_iter().fold(self, |lowering, found| {
            lowering.placed(PlaceRow { subject: None, owner: Some(found) });
            lowering
        });
    }

    pub(super) fn impl_entered(&mut self, block: &ItemImpl) -> Descend {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        self.lowered.rows.impls.push(ImplRow::written(&self.here, self.context.region(), block, &declared));
        let rows = &mut self.lowered.rows.places.subjects;
        let subjected = SubjectId(rows.len());
        let implemented =
            block.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let traited = block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path));
        rows.push(SubjectRow { scope: self.here.scope(), ty: subject, implemented, traited });
        let place = self.placed(PlaceRow { subject: Some(subjected), owner: None });
        self.context = self.context.of_impl(block, place);
        self.enclosed(Enclosure::of_impl(block), generics)
    }
}
