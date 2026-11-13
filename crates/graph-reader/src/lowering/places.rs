use syn::{Attribute, Ident, ItemImpl, ItemMacro, ItemTrait, Macro, Stmt};

use graph::{Building, FileId, ItemDeclaration, ItemId, Region, Resolved, Seat, TypeRef};
use lint_core::{Macros, Syntax};
use lint_layout::{LineNumber, Name, Position};

use crate::namespace::{Domain, PortMacro, Resolving, ScopeId, UsePath};

use super::frames::Frame;
use super::signature::Written;
use super::{Enclosure, ImplRow, Lowering};
use lint_sweep::{Descend, Sweep};

#[derive(Clone, Copy, Debug)]
pub(super) struct PlaceId(usize);

#[derive(Clone, Copy, Debug)]
struct SubjectId(usize);

#[derive(Debug)]
struct SubjectRow {
    scope: ScopeId,
    ty: TypeRef<Written>,
    implemented: Option<Name>,
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
                let implemented =
                    row.subject.and_then(|id| self.subjects.get(id.0)).and_then(|found| found.implemented.clone());
                Place { subject, owner, declared, implemented }
            })
            .collect();
        Places(places)
    }

    fn subject_of(&self, place: Option<PlaceId>) -> Option<SubjectId> {
        place.and_then(|id| self.places.get(id.0)).and_then(|row| row.subject)
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
    }
}

impl Lowering {
    fn placed(&mut self, row: PlaceRow) -> PlaceId {
        let places = &mut self.lowered.rows.places.places;
        let place = PlaceId(places.len());
        places.push(row);
        place
    }

    pub(super) fn owning(&mut self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> PlaceId {
        let subject = self.lowered.rows.places.subject_of(self.context.place());
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner = OwnerRow { scope: self.here.scope(), name: UsePath::from(ident), domain, line, region };
        self.placed(PlaceRow { subject, owner: Some(owner) })
    }

    pub(super) fn owned_by(&mut self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> Descend {
        let frame = self.frame();
        let place = self.owning(ident, domain, attributes);
        self.context = self.context.owned(place);
        self.saved(frame)
    }

    pub(super) fn macro_left(&mut self, item: &ItemMacro) -> &mut Self {
        match &item.ident {
            Some(_) => self.transcribed(&Macros::statements(&item.mac)),
            None => self.expanded(&item.mac),
        }
    }

    fn transcribed(&mut self, statements: &[Stmt]) -> &mut Self {
        let outer = self.context;
        self.context = self.context.transcribed();
        statements.iter().filter_map(Syntax::stmt_item).fold(&mut *self, |inner, declared| {
            Sweep::through_item(declared, inner);
            inner
        });
        self.context = outer;
        self
    }

    fn expanded(&mut self, invocation: &Macro) -> &mut Self {
        Macros::items(invocation).iter().fold(self, |lowering, declared| {
            Sweep::through_item(declared, lowering);
            lowering
        })
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

    pub(super) fn impl_entered(&mut self, block: &ItemImpl, frame: Frame) -> Descend {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        self.lowered.rows.impls.push(ImplRow::written(&self.here, self.context.region(), block, &declared));
        let rows = &mut self.lowered.rows.places.subjects;
        let subjected = SubjectId(rows.len());
        let implemented =
            block.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        rows.push(SubjectRow { scope: self.here.scope(), ty: subject, implemented });
        let place = self.placed(PlaceRow { subject: Some(subjected), owner: None });
        self.context = self.context.of_impl(block, place);
        self.enclosed(Enclosure::of_impl(block), generics, frame)
    }
}
