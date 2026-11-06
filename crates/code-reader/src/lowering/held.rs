use syn::spanned::Spanned;
use syn::{Attribute, Field, Generics, Ident, ItemStruct};

use code_model::{Building, Held, Holding, ItemId, Slot};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::seat::{Built, Located};
use super::signature::{InScope, Written};
use super::{ImplRow, Lowering};
use code_model::{Region, TypeRef};

#[derive(Clone, Debug)]
pub(super) struct Holder {
    line: LineNumber,
    name: Name,
    held: Held,
}

impl Holder {
    pub(super) fn named(ident: &Ident, held: Held) -> Holder {
        Holder { line: LineNumber::from(ident.span()), name: Name::from(ident), held }
    }

    fn field(&self, field: &Field) -> Holder {
        let name = field.ident.as_ref().map(Name::from).unwrap_or_else(|| self.name.clone());
        Holder { line: LineNumber::from(field.span()), name, held: self.held }
    }
}

#[derive(Debug)]
pub(super) struct SlotRow {
    subject: TypeRef<Written>,
    interface: Option<Written>,
}

impl SlotRow {
    pub(super) fn new(subject: TypeRef<Written>, interface: Option<Written>) -> SlotRow {
        SlotRow { subject, interface }
    }

    fn built(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Slot {
        let interface = self.interface.map(|interface| interface.sited(resolving, scope));
        Slot::new(Written::typed(self.subject, resolving, scope), interface)
    }
}

#[derive(Debug)]
pub(super) enum Link {
    Free,
    Declares(UsePath),
    Slot(SlotRow),
}

impl From<Option<UsePath>> for Link {
    fn from(declared: Option<UsePath>) -> Link {
        match declared {
            Some(name) => Link::Declares(name),
            None => Link::Free,
        }
    }
}

impl From<Option<SlotRow>> for Link {
    fn from(slot: Option<SlotRow>) -> Link {
        match slot {
            Some(slot) => Link::Slot(slot),
            None => Link::Free,
        }
    }
}

impl Link {
    fn declares(&self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Option<ItemId> {
        match self {
            Link::Declares(name) => resolving.path(scope, name, Domain::Types).crate_item(),
            Link::Free | Link::Slot(_) => None,
        }
    }

    fn slotted(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Option<Slot> {
        match self {
            Link::Slot(slot) => Some(slot.built(resolving, scope)),
            Link::Free | Link::Declares(_) => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct HoldingRow {
    located: Located,
    holder: Holder,
    region: Region,
    link: Link,
    ty: TypeRef<Written>,
}

impl Built for HoldingRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        let scope = self.located.scope();
        let declares = self.link.declares(resolving, scope);
        let slot = self.link.slotted(resolving, scope);
        let holding = Holding::new(self.holder.name, self.holder.held, Written::typed(self.ty, resolving, scope))
            .placed(self.region, declares, slot);
        self.located.holding(building, holding)
    }
}

impl Lowering<'_, '_> {
    pub(super) fn hold(&mut self, holder: Holder, link: Link, ty: TypeRef<Written>) {
        let located = self.here.located(holder.line);
        self.lowered.holdings.push(HoldingRow { located, holder, region: self.context.region(), link, ty });
    }

    pub(super) fn derived(&mut self, attributes: &[Attribute], ident: &Ident) {
        let rows = ImplRow::derives(&self.here, self.context.region(), attributes, ident);
        self.lowered.rows.extend(rows);
    }

    pub(super) fn parameters(&mut self, holder: Holder, generics: &Generics, scope: &InScope) {
        scope.constraints(generics).into_iter().for_each(|ty| self.hold(holder.clone(), Link::Free, ty));
    }

    pub(super) fn struct_held(&mut self, item: &ItemStruct, scope: &InScope, link: Link) {
        match Syntax::newtype_field(&item.fields) {
            Some(field) => self.hold(Holder::named(&item.ident, Held::NewtypeField), link, scope.lowered(&field.ty)),
            None => self.fields_held(&Holder::named(&item.ident, Held::StructField), item.fields.iter(), scope),
        }
    }

    pub(super) fn fields_held<'field>(
        &mut self,
        holder: &Holder,
        fields: impl Iterator<Item = &'field Field>,
        scope: &InScope,
    ) {
        fields.for_each(|field| self.hold(holder.field(field), Link::Free, scope.lowered(&field.ty)));
    }
}
