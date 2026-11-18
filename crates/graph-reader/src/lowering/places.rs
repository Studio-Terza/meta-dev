use super::walk::Lowering;
use super::{Frame, Framed, Grain, Grains, Owning};
use crate::namespace::ScopeOp;
use lint_sweep::{Contributed, Contributing, WalkPosition};
use std::collections::BTreeMap;
use std::sync::Arc;

use syntax::Spanned;
use syntax::{Attribute, Ident, ItemImpl, ItemMacro, ItemTrait, Stmt};

use algebra::{Monoid, Semigroup};
use graph::{FileId, ItemDeclaration, ItemId, Mention, ModelPart, Region, Resolved, Seat, TypeRef};
use lint_core::{Macros, Syntax};
use lint_layout::{LineNumber, Name, Position};

use crate::namespace::{
    Domain, Entries, KeyRole, NodeKey, PortMacro, Resolving, ScopeKey, ScopeMap, TranscribedRow, UsePath,
};

use super::regions::MacroItems;
use super::signature::Written;
use super::{Enclosure, ImplRow};
use lint_sweep::Descend;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PlaceId(ScopeKey);

#[derive(Debug)]
struct SubjectRow {
    key: ScopeKey,
    scope: ScopeKey,
    ty: TypeRef<Written>,
    implemented: Option<Name>,
    traited: Option<UsePath>,
}

impl SubjectRow {
    fn implemented_trait(&self, resolving: &Resolving<'_, '_, '_>, scopes: &ScopeMap) -> Option<Resolved> {
        self.traited.as_ref().map(|path| resolving.path(scopes.scope(&self.scope), path, Domain::Types))
    }
}

#[derive(Debug)]
struct OwnerRow {
    scope: ScopeKey,
    name: UsePath,
    domain: Domain,
    line: LineNumber,
    region: Region,
}

impl OwnerRow {
    fn settled(&self, resolving: &Resolving<'_, '_, '_>, scopes: &ScopeMap) -> Option<ItemId> {
        resolving.path(scopes.scope(&self.scope), &self.name, self.domain).crate_item()
    }
}

#[derive(Debug)]
struct PlaceRow {
    key: PlaceId,
    subject: Option<ScopeKey>,
    owner: Option<OwnerRow>,
}

#[derive(Debug, Default)]
pub(super) struct PlaceRows {
    subjects: Vec<SubjectRow>,
    places: Vec<PlaceRow>,
}

impl Semigroup for PlaceRows {
    fn combine(self, other: PlaceRows) -> PlaceRows {
        PlaceRows { subjects: self.subjects.combine(other.subjects), places: self.places.combine(other.places) }
    }
}

impl Monoid for PlaceRows {}

#[derive(Debug)]
struct Subjected<'rows> {
    ty: TypeRef,
    row: &'rows SubjectRow,
}

impl PlaceRows {
    pub(super) fn settled(&self, resolving: &Resolving<'_, '_, '_>, scopes: &ScopeMap) -> Places {
        let subjects: BTreeMap<&ScopeKey, Subjected<'_>> = self
            .subjects
            .iter()
            .map(|row| (&row.key, Subjected { ty: Written::typed(&row.ty, resolving, scopes.scope(&row.scope)), row }))
            .collect();
        let places: BTreeMap<PlaceId, Place> = self
            .places
            .iter()
            .map(|row| {
                let subjected = row.subject.as_ref().and_then(|key| subjects.get(key));
                let subject = subjected.map(|found| found.ty.clone());
                let declared = row.owner.as_ref().and_then(|owner| {
                    let item = owner.settled(resolving, scopes)?;
                    Some(Declared { line: owner.line, item, region: owner.region })
                });
                let owner = declared
                    .map(|found| found.item)
                    .or_else(|| subject.as_ref()?.named().map(AsRef::<Resolved>::as_ref)?.crate_item());
                let implemented = subjected.and_then(|found| found.row.implemented.clone());
                let traited = subjected.and_then(|found| found.row.implemented_trait(resolving, scopes));
                (row.key.clone(), Place { subject, owner, declared, implemented, traited })
            })
            .collect();
        let order = self.places.iter().map(|row| row.key.clone()).collect();
        Places { order, places }
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
pub(super) struct Places {
    order: Vec<PlaceId>,
    places: BTreeMap<PlaceId, Place>,
}

impl Places {
    pub(super) fn placed(&self, file: FileId) -> ModelPart {
        Monoid::concat(self.order.iter().filter_map(|id| self.places.get(id)?.declared).map(|declared| {
            ModelPart::placed(file, declared.line, ItemDeclaration::new(declared.item, declared.region))
        }))
    }

    pub(super) fn seat(&self, region: Region, place: Option<&PlaceId>) -> Seat {
        let found = place.and_then(|id| self.places.get(id));
        Seat::new(region, found.and_then(|seated| seated.subject.clone()))
            .owned(found.and_then(|seated| seated.owner))
            .implementing(found.and_then(|seated| seated.implemented.clone()))
            .traited(found.and_then(|seated| seated.traited.clone()))
    }
}

impl<'read> Frame<'read> {
    fn placed(&self, role: KeyRole, seed: PlaceSeed) -> Owning {
        let place = PlaceId(ScopeKey::reached(&self.within, NodeKey::new(self.at, role)));
        let row = PlaceRow { key: place.clone(), subject: seed.subject, owner: seed.owner };
        let rows = Grain::Place(PlaceRows { places: vec![row], subjects: Vec::new() });
        Owning { place, grains: Grains::from(rows) }
    }

    pub(super) fn owning(&self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> Owning {
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner = OwnerRow { scope: self.here.scope().clone(), name: UsePath::from(ident), domain, line, region };
        self.placed(KeyRole::Owner, PlaceSeed { subject: None, owner: Some(owner) })
    }

    pub(super) fn owned_by(
        &self,
        ident: &Ident,
        domain: Domain,
        attributes: &[Attribute],
        rows: Grains,
    ) -> Contributed<Framed<'read>, Grains> {
        let Owning { place, grains } = self.owning(ident, domain, attributes);
        let inner = Frame { context: self.context.owned(place), ..self.clone() };
        Contributed::new(Framed::framing(inner), rows.combine(grains), Descend::Into)
    }

    pub(super) fn macro_left(&self, item: &ItemMacro) -> Grains {
        match &item.ident {
            Some(ident) => self.rules_defined(item, ident),
            None => self.expanded(&Macros::items_in_file(self.source, &item.mac)),
        }
    }

    fn rules_defined(&self, item: &ItemMacro, ident: &Ident) -> Grains {
        let row = Grains::from(Grain::Transcribed(TranscribedRow::read(self.here.scope().clone(), ident, &item.mac)));
        let anchored = TranscribedRow::anchored(&item.mac);
        let attributed = anchored
            .iter()
            .map(|path| self.used(UsePath::from(path), Position::from(path.span()), Mention::Expanded))
            .fold(row, Semigroup::combine);
        attributed.combine(self.transcribed(&Macros::expanded(self.source, &item.mac)))
    }

    pub(super) fn expansion(&self, items: MacroItems) -> Lowering<'read> {
        let within: Arc<[WalkPosition]> = self.within.iter().copied().chain([self.at]).collect();
        Lowering::below(Frame { context: self.context.within_macro(items), within, ..self.clone() })
    }

    pub(super) fn swept<'item>(lowering: Lowering<'read>, items: impl Iterator<Item = &'item syntax::Item>) -> Grains {
        let contributing = Contributing::new(lowering);
        contributing.contributed_items(items).concluded(|_, grains| grains)
    }

    pub(super) fn transcribed(&self, statements: &[Stmt]) -> Grains {
        Self::swept(self.expansion(MacroItems::Transcribed), statements.iter().filter_map(Syntax::stmt_item))
    }

    pub(super) fn expanded(&self, items: &[syntax::Item]) -> Grains {
        let absorbed = ScopeOp::Absorbed { key: self.here.scope().clone(), entries: Entries::of_items(items) };
        let walked = Self::swept(self.expansion(MacroItems::Expanded), items.iter());
        Grains::from(Grain::Op(absorbed)).combine(walked)
    }

    pub(super) fn trait_owning(&self, ident: &Ident, attributes: &[Attribute]) -> Owning {
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner = OwnerRow {
            scope: self.here.scope().clone(),
            name: UsePath::from(ident),
            domain: Domain::Types,
            line,
            region,
        };
        self.placed(KeyRole::Owner, PlaceSeed { subject: None, owner: Some(owner) })
    }

    pub(super) fn filler_owning(&self, item: &ItemTrait) -> Grains {
        let line = Position::from(item.ident.span()).line();
        let region = self.context.within_item(&item.attrs).region();
        let owner = PortMacro::marked(item).map(|found| OwnerRow {
            scope: self.here.scope().clone(),
            name: UsePath::suffixed(&found.ident, PortMacro::FILLER),
            domain: Domain::Types,
            line,
            region,
        });
        owner.map_or_else(Grains::default, |found| {
            self.placed(KeyRole::Filler, PlaceSeed { subject: None, owner: Some(found) }).grains
        })
    }

    pub(super) fn impl_entered(&self, block: &ItemImpl) -> Contributed<Framed<'read>, Grains> {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        let implemented = ImplRow::written(&self.here, self.context.region(), block, &declared);
        let key = ScopeKey::reached(&self.within, NodeKey::new(self.at, KeyRole::Scope));
        let traited = block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path));
        let named =
            block.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let subjected =
            SubjectRow { key: key.clone(), scope: self.here.scope().clone(), ty: subject, implemented: named, traited };
        let Owning { place, grains } = self.placed(KeyRole::Subject, PlaceSeed { subject: Some(key), owner: None });
        let rows: Grains = Monoid::concat(
            [Grain::Impl(implemented), Grain::Place(PlaceRows { subjects: vec![subjected], places: Vec::new() })]
                .into_iter()
                .map(Grains::from),
        );
        let inner = Frame {
            context: self.context.of_impl(block, place),
            enclosure: Enclosure::of_impl(block),
            generics: Arc::new(generics),
            ..self.clone()
        };
        Contributed::new(Framed::framing(inner), rows.combine(grains), Descend::Into)
    }
}

#[derive(Debug)]
struct PlaceSeed {
    subject: Option<ScopeKey>,
    owner: Option<OwnerRow>,
}
