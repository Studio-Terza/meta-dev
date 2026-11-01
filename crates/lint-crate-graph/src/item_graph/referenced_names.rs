use std::collections::BTreeMap;

use quote::ToTokens;
use syn::{Attribute, Item, Type};

use lint_core::Name;
use lint_core::Walk;

use crate::item_graph::scope::Scope;

#[derive(Debug, Default)]
pub struct ReferencedNames(BTreeMap<Name, Vec<Name>>);

impl ReferencedNames {
    pub fn of_items(items: &[Item], scope: &Scope) -> ReferencedNames {
        items.iter().fold(ReferencedNames::default(), |names, item| names.with_item(item, scope))
    }

    pub fn into_map(self) -> BTreeMap<Name, Vec<Name>> {
        self.0
    }

    fn with_item(mut self, item: &Item, scope: &Scope) -> ReferencedNames {
        let (attributes, owner, nested): (&[Attribute], Option<Name>, &[Item]) = match item {
            Item::Struct(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Enum(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Trait(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Fn(item) => (&item.attrs, Some(Name::from(&item.sig.ident)), &[]),
            Item::Type(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Const(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Static(item) => (&item.attrs, Some(Name::from(&item.ident)), &[]),
            Item::Impl(item) => (&item.attrs, Self::self_type(&item.self_ty), &[]),
            Item::Mod(module) => (&module.attrs, None, module.content.as_ref().map_or(&[], |(_, items)| items)),
            _ => (&[], None, &[]),
        };
        owner
            .into_iter()
            .map(|owner| (owner, scope.referenced(item.to_token_stream())))
            .chain(Self::of_items(nested, scope).0)
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .fold(&mut self.0, |uses, (owner, found)| {
                uses.entry(owner).or_default().extend(found);
                uses
            });
        self
    }

    fn self_type(self_type: &Type) -> Option<Name> {
        match self_type {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }
}
