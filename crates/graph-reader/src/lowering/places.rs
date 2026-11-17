use super::FileRows;
use super::walk::Lowering;
use super::{Frame, Framed, Grains, Owning};
use crate::namespace::ScopeOp;
use lint_sweep::{Address, Contributed, Contributing};
use std::collections::BTreeMap;
use std::sync::Arc;

use syntax::{Attribute, Ident, ItemImpl, ItemMacro, ItemTrait, Macro, Stmt};

use algebra::{Monoid, Semigroup};
use graph::{Building, FileId, ItemDeclaration, ItemId, Region, Resolved, Seat, TypeRef};
use lint_core::{Macros, Syntax};
use lint_layout::{LineNumber, Name, Position};

use crate::namespace::{Domain, Entries, PortMacro, Resolving, ScopeKey, ScopeMap, UsePath};

use super::regions::MacroItems;
use super::signature::Written;
use super::{Enclosure, ImplRow};
use lint_sweep::{Descend, Sweep};

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

impl PlaceRows {
    pub(super) fn settled(&self, resolving: &Resolving<'_, '_, '_>, scopes: &ScopeMap) -> Places {
        let subjects: BTreeMap<&ScopeKey, (TypeRef, &SubjectRow)> = self
            .subjects
            .iter()
            .map(|row| (&row.key, (Written::typed(&row.ty, resolving, scopes.scope(&row.scope)), row)))
            .collect();
        let places: Vec<Place> = self
            .places
            .iter()
            .map(|row| {
                let subjected = row.subject.as_ref().and_then(|key| subjects.get(key));
                let subject = subjected.map(|(ty, _)| ty.clone());
                let declared = row.owner.as_ref().and_then(|owner| {
                    let item = owner.settled(resolving, scopes)?;
                    Some(Declared { line: owner.line, item, region: owner.region })
                });
                let owner = declared
                    .map(|found| found.item)
                    .or_else(|| subject.as_ref()?.named().map(AsRef::<Resolved>::as_ref)?.crate_item());
                let implemented = subjected.and_then(|(_, found)| found.implemented.clone());
                let traited = subjected.and_then(|(_, found)| found.implemented_trait(resolving, scopes));
                Place { subject, owner, declared, implemented, traited }
            })
            .collect();
        let index = self.places.iter().enumerate().map(|(position, row)| (row.key.clone(), position)).collect();
        Places { order: places, index }
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
    order: Vec<Place>,
    index: BTreeMap<PlaceId, usize>,
}

impl Places {
    pub(super) fn placed(&self, file: FileId, building: Building) -> Building {
        self.order.iter().filter_map(|place| place.declared).fold(building, |mut built, declared| {
            built.placed(file, declared.line, ItemDeclaration::new(declared.item, declared.region));
            built
        })
    }

    pub(super) fn seat(&self, region: Region, place: Option<&PlaceId>) -> Seat {
        let found = place.and_then(|id| self.index.get(id)).and_then(|position| self.order.get(*position));
        Seat::new(region, found.and_then(|seated| seated.subject.clone()))
            .owned(found.and_then(|seated| seated.owner))
            .implementing(found.and_then(|seated| seated.implemented.clone()))
            .traited(found.and_then(|seated| seated.traited.clone()))
    }
}

impl<'read> Frame<'read> {
    fn placed(&self, key: Address, seed: PlaceSeed) -> Owning {
        let place = PlaceId(self.key(key));
        let row = PlaceRow { key: place.clone(), subject: seed.subject, owner: seed.owner };
        let rows = FileRows { places: PlaceRows { places: vec![row], subjects: Vec::new() }, ..FileRows::default() };
        Owning { place, grains: Grains::from(rows) }
    }

    pub(super) fn owning(&self, ident: &Ident, domain: Domain, attributes: &[Attribute]) -> Owning {
        let line = Position::from(ident.span()).line();
        let region = self.context.within_item(attributes).region();
        let owner = OwnerRow { scope: self.here.scope().clone(), name: UsePath::from(ident), domain, line, region };
        self.placed(Address::from(ident), PlaceSeed { subject: None, owner: Some(owner) })
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
            Some(_) => self.transcribed(item, &Macros::expanded(self.source, &item.mac)),
            None => self.expanded(item, &item.mac),
        }
    }

    fn expansion(&self, item: &ItemMacro, items: MacroItems) -> Lowering<'read> {
        let within: Arc<[Address]> = self.within.iter().copied().chain([Address::from(item)]).collect();
        Lowering::below(Frame { context: self.context.within_macro(items), within, ..self.clone() })
    }

    fn swept<'item>(lowering: Lowering<'read>, items: impl Iterator<Item = &'item syntax::Item>) -> Grains {
        let mut contributing = Contributing::new(lowering);
        items.for_each(|declared| Sweep::expanded_item(declared, &mut contributing));
        contributing.concluded(|_, _, grains| grains)
    }

    fn transcribed(&self, item: &ItemMacro, statements: &[Stmt]) -> Grains {
        Self::swept(self.expansion(item, MacroItems::Transcribed), statements.iter().filter_map(Syntax::stmt_item))
    }

    fn expanded(&self, item: &ItemMacro, invocation: &Macro) -> Grains {
        let items = Macros::items_in_file(self.source, invocation);
        let absorbed = ScopeOp::Absorbed { key: self.here.scope().clone(), entries: Entries::of_items(&items) };
        let walked = Self::swept(self.expansion(item, MacroItems::Expanded), items.iter());
        Grains { ops: vec![absorbed], ..Grains::default() }.combine(walked)
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
        self.placed(Address::from(ident), PlaceSeed { subject: None, owner: Some(owner) })
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
            self.placed(Address::from(item), PlaceSeed { subject: None, owner: Some(found) }).grains
        })
    }

    pub(super) fn impl_entered(&self, block: &ItemImpl) -> Contributed<Framed<'read>, Grains> {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        let implemented = ImplRow::written(&self.here, self.context.region(), block, &declared);
        let key = self.key(Address::from(block));
        let traited = block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path));
        let named =
            block.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let subjected =
            SubjectRow { key: key.clone(), scope: self.here.scope().clone(), ty: subject, implemented: named, traited };
        let Owning { place, grains } =
            self.placed(Address::from(&block.self_ty), PlaceSeed { subject: Some(key), owner: None });
        let rows = FileRows {
            impls: vec![implemented],
            places: PlaceRows { subjects: vec![subjected], places: Vec::new() },
            ..FileRows::default()
        };
        let inner = Frame {
            context: self.context.of_impl(block, place),
            enclosure: Enclosure::of_impl(block),
            generics: Arc::new(generics),
            ..self.clone()
        };
        Contributed::new(Framed::framing(inner), Grains::from(rows).combine(grains), Descend::Into)
    }
}

#[derive(Debug)]
struct PlaceSeed {
    subject: Option<ScopeKey>,
    owner: Option<OwnerRow>,
}
