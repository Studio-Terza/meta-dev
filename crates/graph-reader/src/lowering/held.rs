use super::{Frame, Framed, Grain, Grains};
use algebra::{Monoid, Semigroup};
use lint_sweep::Contributed;
use syntax::Spanned;
use syntax::{Attribute, Field, Generics, Ident, ItemStruct, ItemTrait, Type};

use graph::{Held, Holding, ItemId, ModelPart};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::ImplRow;
use super::attributes::Attributes;
use super::seat::{Footing, Located, Lowers};
use super::signature::{InScope, Written};
use graph::{Region, TypeRef};

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
pub(super) struct Link {
    name: UsePath,
    domain: Domain,
}

impl Link {
    pub(super) fn declaring(ident: &Ident) -> Link {
        Link { name: UsePath::from(ident), domain: Domain::Types }
    }

    pub(super) fn valued(self) -> Link {
        Link { domain: Domain::Values, ..self }
    }

    fn declares(&self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Option<ItemId> {
        resolving.path(scope, &self.name, self.domain).crate_item()
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

impl Lowers for HoldingRow {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        let scope = footing.scopes().scope(self.located.scope());
        let resolving = &footing.resolving();
        let declares = self.link.declares(resolving, scope);
        let holding = Holding::new(self.holder.name, self.holder.held, Written::typed(&self.ty, resolving, scope))
            .placed(self.region.combine(footing.outer()), declares);
        self.located.holding(footing, holding)
    }
}

impl<'read> Frame<'read> {
    pub(super) fn valued(
        &self,
        ident: &Ident,
        ty: &Type,
        attributes: &[Attribute],
    ) -> Contributed<Framed<'read>, Grains> {
        let target = self.generics.lowered(ty);
        let held = self.hold(Holder::named(ident, Held::Value), Link::declaring(ident).valued(), target);
        self.owned_by(ident, Domain::Values, attributes, held)
    }

    pub(super) fn hold(&self, holder: Holder, link: Link, ty: TypeRef<Written>) -> Grains {
        let located = self.here.located(holder.position);
        let row = HoldingRow { located, holder, region: self.context.region(), link, ty };
        Grains::from(Grain::Holding(row))
    }

    pub(super) fn attributed(&self, attributes: &Attributes, ident: &Ident) -> Grains {
        let rows = ImplRow::derives(&self.here, self.context.region(), attributes, ident);
        Monoid::concat(rows.into_iter().map(Grain::Impl).map(Grains::from))
    }

    pub(super) fn parameters(&self, holder: &Holder, generics: &Generics, link: &Link) -> Grains {
        let scope = InScope::clone(&self.generics).combine(InScope::from(generics));
        Monoid::concat(scope.constraints(generics).into_iter().map(|ty| self.hold(holder.clone(), link.clone(), ty)))
    }

    pub(super) fn trait_bounds(&self, item: &ItemTrait, scope: &InScope) -> Grains {
        let holder = Holder::named(&item.ident, Held::Bound);
        let link = Link::declaring(&item.ident);
        let associated =
            item.items.iter().filter_map(Syntax::trait_type).map(|declared| scope.bounded(declared.bounds.iter()));
        let parameters = self.parameters(&holder, &item.generics, &link);
        let bounds = [scope.bounded(item.supertraits.iter())]
            .into_iter()
            .chain(associated)
            .map(|ty| self.hold(holder.clone(), link.clone(), ty));
        parameters.combine(Monoid::concat(bounds))
    }

    pub(super) fn struct_held(&self, item: &ItemStruct, scope: &InScope, link: &Link) -> Grains {
        Syntax::newtype_field(&item.fields).map_or_else(
            || self.fields_held(&Holder::named(&item.ident, Held::StructField), link, item.fields.iter(), scope),
            |field| self.hold(Holder::named(&item.ident, Held::NewtypeField), link.clone(), scope.lowered(&field.ty)),
        )
    }

    pub(super) fn fields_held<'field>(
        &self,
        holder: &Holder,
        link: &Link,
        fields: impl Iterator<Item = &'field Field>,
        scope: &InScope,
    ) -> Grains {
        Monoid::concat(fields.map(|field| self.hold(holder.field(field), link.clone(), scope.lowered(&field.ty))))
    }
}
