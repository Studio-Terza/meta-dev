mod declared;
mod macro_items;

use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::Item;

use lint_core::SourceFile;
use lint_layout::Name;

use crate::ItemGraph;
use crate::scope::Scope;
use declared::Declared;
use macro_items::MacroItems;

#[derive(Debug, Default)]
pub struct ReferencedNames {
    declared: BTreeMap<Name, Vec<Name>>,
    implemented: BTreeMap<Name, Vec<Name>>,
    stand_ins: BTreeSet<Name>,
}

impl ReferencedNames {
    const UNNAMED: &str = "_";

    #[must_use]
    pub fn of_file(file: &SourceFile) -> ReferencedNames {
        Self::of_items(&file.syntax().items, &Scope::of_file(file.syntax()))
    }

    #[must_use]
    pub fn into_uses(self) -> BTreeMap<Name, BTreeSet<Name>> {
        let all = Self::united(self.declared, self.implemented);
        all.into_iter().map(|(owner, uses)| (owner, uses.into_iter().collect())).collect()
    }

    pub(crate) fn of_items(items: &[Item], scope: &Scope<'_>) -> ReferencedNames {
        items.iter().fold(ReferencedNames::default(), |names, item| names.with_item(item, scope))
    }

    pub(crate) fn merged(self, other: ReferencedNames) -> ReferencedNames {
        ReferencedNames {
            declared: Self::united(self.declared, other.declared),
            implemented: Self::united(self.implemented, other.implemented),
            stand_ins: self.stand_ins.into_iter().chain(other.stand_ins).collect(),
        }
    }

    pub(crate) fn into_graph(self) -> ItemGraph {
        let ReferencedNames { declared, implemented, stand_ins } = self;
        let foreign = implemented.into_iter().filter(|(owner, _)| declared.contains_key(owner)).collect();
        ItemGraph::of_references(Self::united(declared, foreign)).standing_in(&stand_ins)
    }

    fn united(all: BTreeMap<Name, Vec<Name>>, more: BTreeMap<Name, Vec<Name>>) -> BTreeMap<Name, Vec<Name>> {
        more.into_iter().fold(all, |mut all, (owner, uses)| {
            all.entry(owner).or_default().extend(uses);
            all
        })
    }

    fn with_item(self, item: &Item, scope: &Scope<'_>) -> ReferencedNames {
        match Declared::from(item).entered() {
            Some(declared) => self.merged(declared.referenced(scope)),
            None => self,
        }
    }

    fn standing_in(item: &Item) -> Option<Name> {
        match item {
            Item::Type(alias) => Some(Name::from(&alias.ident)),
            Item::Const(constant) => Some(Name::from(&constant.ident)),
            Item::Macro(call) => MacroItems::defined_by(call),
            _ => None,
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
