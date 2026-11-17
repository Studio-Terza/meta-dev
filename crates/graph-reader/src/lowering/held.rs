use syntax::Spanned;
use syntax::{Attribute, Field, Generics, Ident, ItemEnum, ItemStruct, ItemTrait, ItemType, ItemUnion, Type};

use graph::{Building, Held, Holding, ItemId};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::attributes::Attributes;
use super::seat::{Built, Footing, Located};
use super::signature::{InScope, Written};
use super::{AliasRow, ImplRow, Lowering};
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

impl Built for HoldingRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let scope = self.located.scope();
        let resolving = &footing.resolving();
        let declares = self.link.declares(resolving, scope);
        let holding = Holding::new(self.holder.name, self.holder.held, Written::typed(&self.ty, resolving, scope))
            .placed(self.region.joined(footing.outer()), declares);
        self.located.holding(footing, building, holding)
    }
}

impl<'read> Lowering<'read> {
    pub(super) fn valued(self: Box<Self>, ident: &Ident, ty: &Type, attributes: &[Attribute]) -> Box<Lowering<'read>> {
        let target = self.generics.lowered(ty);
        self.hold(Holder::named(ident, Held::Value), Link::declaring(ident).valued(), target).owned_by(
            ident,
            Domain::Values,
            attributes,
        )
    }

    pub(super) fn hold(mut self: Box<Self>, holder: Holder, link: Link, ty: TypeRef<Written>) -> Box<Lowering<'read>> {
        let located = self.here.located(holder.position);
        let region = self.context.region();
        self.lowered.rows.holdings.push(HoldingRow { located, holder, region, link, ty });
        self
    }

    pub(super) fn attributed(mut self: Box<Self>, attributes: &Attributes, ident: &Ident) -> Box<Lowering<'read>> {
        let rows = ImplRow::derives(&self.here, self.context.region(), attributes, ident);
        self.lowered.rows.impls.extend(rows);
        self
    }

    pub(super) fn parameters(
        self: Box<Self>,
        holder: &Holder,
        generics: &Generics,
        link: &Link,
    ) -> Box<Lowering<'read>> {
        let scope = self.generics.extended(generics);
        scope
            .constraints(generics)
            .into_iter()
            .fold(self, |lowering, ty| lowering.hold(holder.clone(), link.clone(), ty))
    }

    pub(super) fn trait_bounds(self: Box<Self>, item: &ItemTrait, scope: &InScope) -> Box<Lowering<'read>> {
        let holder = Holder::named(&item.ident, Held::Bound);
        let link = Link::declaring(&item.ident);
        let associated =
            item.items.iter().filter_map(Syntax::trait_type).map(|declared| scope.bounded(declared.bounds.iter()));
        let bounded = self.parameters(&holder, &item.generics, &link);
        [scope.bounded(item.supertraits.iter())]
            .into_iter()
            .chain(associated)
            .fold(bounded, |lowering, ty| lowering.hold(holder.clone(), link.clone(), ty))
    }

    pub(super) fn struct_held(self: Box<Self>, item: &ItemStruct, scope: &InScope, link: Link) -> Box<Lowering<'read>> {
        match Syntax::newtype_field(&item.fields) {
            Some(field) => self.hold(Holder::named(&item.ident, Held::NewtypeField), link, scope.lowered(&field.ty)),
            None => self.fields_held(&Holder::named(&item.ident, Held::StructField), &link, item.fields.iter(), scope),
        }
    }

    pub(super) fn fields_held<'field>(
        self: Box<Self>,
        holder: &Holder,
        link: &Link,
        fields: impl Iterator<Item = &'field Field>,
        scope: &InScope,
    ) -> Box<Lowering<'read>> {
        fields.fold(self, |lowering, field| lowering.hold(holder.field(field), link.clone(), scope.lowered(&field.ty)))
    }

    pub(super) fn aliased(self: Box<Self>, item: &ItemType) -> Box<Lowering<'read>> {
        let generics = InScope::outside().extended(&item.generics);
        let target = generics.lowered(&item.ty);
        let alias = AliasRow {
            scope: self.here.scope(),
            name: UsePath::from(&item.ident),
            parameters: generics.declarations(&item.generics),
            target: target.clone(),
        };
        let mut held = self.hold(Holder::named(&item.ident, Held::Alias), Link::declaring(&item.ident), target);
        held.lowered.rows.aliases.push(alias);
        held.owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    pub(super) fn struct_entered(self: Box<Self>, item: &ItemStruct) -> Box<Lowering<'read>> {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident)
            .parameters(&Holder::named(&item.ident, Held::StructParameter), &item.generics, &link)
            .struct_held(item, &scope, link)
            .owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    pub(super) fn enum_entered(self: Box<Self>, item: &ItemEnum) -> Box<Lowering<'read>> {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        let opened = self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident).parameters(
            &Holder::named(&item.ident, Held::Bound),
            &item.generics,
            &link,
        );
        item.variants
            .iter()
            .fold(opened, |varied, variant| {
                let holder = Holder::named(&variant.ident, Held::EnumField);
                varied
                    .hold(Holder::named(&variant.ident, Held::Variant), link.clone(), TypeRef::Tuple(Vec::new()))
                    .fields_held(&holder, &link, variant.fields.iter(), &scope)
            })
            .owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    pub(super) fn union_entered(self: Box<Self>, item: &ItemUnion) -> Box<Lowering<'read>> {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident)
            .fields_held(&Holder::named(&item.ident, Held::UnionField), &link, item.fields.named.iter(), &scope)
            .owned_by(&item.ident, Domain::Types, &item.attrs)
    }
}
