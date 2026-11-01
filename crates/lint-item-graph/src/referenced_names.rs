mod macro_items;

use std::collections::BTreeMap;

use quote::ToTokens;
use syn::{Attribute, Item, Type};

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

    pub fn of_items(items: &[Item], scope: &Scope) -> ReferencedNames {
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

    fn with_item(self, item: &Item, scope: &Scope) -> ReferencedNames {
        let (attributes, owner, nested): (&[Attribute], Option<Name>, Vec<Item>) = match item {
            Item::Struct(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Enum(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Trait(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Fn(item) => (&item.attrs, Some(Name::from(&item.sig.ident)), Vec::new()),
            Item::Type(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Const(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Static(item) => (&item.attrs, Some(Name::from(&item.ident)), Vec::new()),
            Item::Impl(item) => (&item.attrs, None, Vec::new()),
            Item::Mod(module) => {
                (&module.attrs, None, module.content.iter().flat_map(|(_, items)| items.clone()).collect())
            }
            Item::Macro(call) => (&call.attrs, None, MacroItems::declared_by(call)),
            _ => (&[], None, Vec::new()),
        };
        match Walk::outside_tests(attributes) {
            Walk::Skipped => self,
            Walk::Entered => {
                let own = match item {
                    Item::Impl(item) => ReferencedNames {
                        implemented: Self::named(Self::self_type(&item.self_ty), scope, item),
                        ..ReferencedNames::default()
                    },
                    _ => ReferencedNames { declared: Self::named(owner, scope, item), ..ReferencedNames::default() },
                };
                self.merged(own).merged(Self::of_items(&nested, scope))
            }
        }
    }

    fn named(owner: Option<Name>, scope: &Scope, item: &impl ToTokens) -> BTreeMap<Name, Vec<Name>> {
        owner
            .filter(|owner| *owner != Self::UNNAMED)
            .map(|owner| (owner, scope.referenced(item.to_token_stream())))
            .into_iter()
            .collect()
    }

    fn self_type(self_type: &Type) -> Option<Name> {
        match self_type {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }
}
