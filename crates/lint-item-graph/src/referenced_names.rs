mod macro_items;

use std::borrow::Cow;
use std::collections::BTreeMap;

use quote::ToTokens;
use syn::{Attribute, Item};

use lint_core::Walk;
use lint_layout::Name;

use crate::scope::Scope;
use macro_items::MacroItems;

#[derive(Debug, Default)]
pub struct ReferencedNames {
    declared: BTreeMap<Name, Vec<Name>>,
    implemented: BTreeMap<Name, Vec<Name>>,
}

impl ReferencedNames {
    const UNNAMED: &str = "_";

    pub fn of_items(items: &[Item], scope: &Scope<'_>) -> ReferencedNames {
        items.iter().fold(ReferencedNames::default(), |names, item| names.with_item(item, scope))
    }

    pub fn merged(self, other: ReferencedNames) -> ReferencedNames {
        ReferencedNames {
            declared: Self::united(self.declared, other.declared),
            implemented: Self::united(self.implemented, other.implemented),
        }
    }

    pub fn into_map(self) -> BTreeMap<Name, Vec<Name>> {
        let ReferencedNames { declared, implemented } = self;
        let foreign = implemented.into_iter().filter(|(owner, _)| declared.contains_key(owner)).collect();
        Self::united(declared, foreign)
    }

    fn united(all: BTreeMap<Name, Vec<Name>>, more: BTreeMap<Name, Vec<Name>>) -> BTreeMap<Name, Vec<Name>> {
        more.into_iter().fold(all, |mut all, (owner, uses)| {
            all.entry(owner).or_default().extend(uses);
            all
        })
    }

    fn with_item(self, item: &Item, scope: &Scope<'_>) -> ReferencedNames {
        let (attributes, owner, nested): (&[Attribute], Option<Name>, Cow<'_, [Item]>) = match item {
            Item::Struct(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Enum(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Trait(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Fn(item) => (&item.attrs, Some(Name::from(&item.sig.ident)), Cow::Borrowed(&[])),
            Item::Type(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Const(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Static(item) => (&item.attrs, Some(Name::from(&item.ident)), Cow::Borrowed(&[])),
            Item::Impl(item) => (&item.attrs, None, Cow::Borrowed(&[])),
            Item::Mod(module) => (
                &module.attrs,
                None,
                module.content.as_ref().map_or(Cow::Borrowed(&[]), |(_, items)| Cow::Borrowed(items.as_slice())),
            ),
            Item::Macro(call) => (&call.attrs, None, Cow::Owned(MacroItems::declared_by(call))),
            _ => (&[], None, Cow::Borrowed(&[])),
        };
        match Walk::outside_tests(attributes) {
            Walk::Skipped => self,
            Walk::Entered => {
                let own = match item {
                    Item::Impl(item) => ReferencedNames {
                        implemented: Self::named(Name::of_type(&item.self_ty), scope, item),
                        ..ReferencedNames::default()
                    },
                    _ => ReferencedNames { declared: Self::named(owner, scope, item), ..ReferencedNames::default() },
                };
                self.merged(own).merged(Self::of_items(&nested, scope))
            }
        }
    }

    fn named(owner: Option<Name>, scope: &Scope<'_>, item: &impl ToTokens) -> BTreeMap<Name, Vec<Name>> {
        owner
            .filter(|owner| *owner != Self::UNNAMED)
            .map(|owner| (owner, scope.referenced(item.to_token_stream())))
            .into_iter()
            .collect()
    }
}
