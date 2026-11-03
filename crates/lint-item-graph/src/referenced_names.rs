mod declared;

use std::collections::BTreeMap;

use quote::ToTokens;
use syn::Item;

use lint_layout::Name;

use crate::scope::Scope;
use declared::Declared;

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
        let referenced = Declared::from(item).entered().map(|declared| declared.referenced(scope)).unwrap_or_default();
        self.merged(referenced)
    }

    fn named(owner: Option<Name>, scope: &Scope<'_>, item: &impl ToTokens) -> BTreeMap<Name, Vec<Name>> {
        owner
            .filter(|owner| *owner != Self::UNNAMED)
            .map(|owner| (owner, scope.referenced(item.to_token_stream())))
            .into_iter()
            .collect()
    }
}
