use syn::spanned::Spanned;
use syn::{Attribute, Field, Generics, Ident, ItemStruct};

use code_model::{Building, Held, Holding, ItemId};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::seat::{Built, Located};
use super::signature::{InScope, Written};
use super::{ImplRow, Lowering};
use code_model::{Region, TypeRef};

#[derive(Clone, Debug)]
pub(super) struct Holder {
    position: Position,
    name: Name,
    held: Held,
}

impl Holder {
    pub(super) fn named(ident: &Ident, held: Held) -> Holder {
        Holder { position: Position::from(ident.span()), name: Name::from(ident), held }
    }

    fn field(&self, field: &Field) -> Holder {
        Holder { position: Position::from(field.span()), name: self.field_name(field), held: self.held }
    }

    fn field_name(&self, field: &Field) -> Name {
        match &field.ident {
            Some(ident) => Name::from(ident),
            None => self.name.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Link {
    Free,
    Declares(UsePath),
}

impl Link {
    fn declares(&self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Option<ItemId> {
        match self {
            Link::Declares(name) => resolving.path(scope, name, Domain::Types).crate_item(),
            Link::Free => None,
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
        let holding = Holding::new(self.holder.name, self.holder.held, Written::typed(&self.ty, resolving, scope))
            .placed(self.region, declares);
        self.located.holding(building, holding)
    }
}

impl Lowering<'_, '_> {
    pub(super) fn hold(&mut self, holder: Holder, link: Link, ty: TypeRef<Written>) {
        let located = self.here.located(holder.position);
        self.lowered.holdings.push(HoldingRow { located, holder, region: self.context.region(), link, ty });
    }

    pub(super) fn derived(&mut self, attributes: &[Attribute], ident: &Ident) {
        let rows = ImplRow::derives(&self.here, self.context.region(), attributes, ident);
        self.lowered.rows.extend(rows);
    }

    pub(super) fn parameters(&mut self, holder: &Holder, generics: &Generics, link: &Link) {
        let scope = self.generics.extended(generics);
        scope.constraints(generics).into_iter().fold(self, |lowering, ty| {
            lowering.hold(holder.clone(), link.clone(), ty);
            lowering
        });
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
        fields.fold(self, |lowering, field| {
            lowering.hold(holder.field(field), Link::Free, scope.lowered(&field.ty));
            lowering
        });
    }
}
