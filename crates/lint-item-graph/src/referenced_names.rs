mod kind;
mod macro_items;
mod spoken;

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::{Attribute, Item};

use lint_core::{SourceFile, Walk};
use lint_layout::{Name, UsePath};

use crate::ItemGraph;
use crate::scope::Scope;
use kind::Kind;
use macro_items::MacroItems;
use spoken::Spoken;

#[derive(Debug, Default)]
pub struct ReferencedNames {
    declared: BTreeMap<Name, Vec<UsePath>>,
    implemented: BTreeMap<Name, Vec<UsePath>>,
    stand_ins: BTreeSet<Name>,
    types: BTreeSet<Name>,
}

impl ReferencedNames {
    const UNNAMED: &str = "_";

    #[must_use]
    pub fn of_file(file: &SourceFile) -> ReferencedNames {
        Self::of_items(&file.syntax().items, &Scope::of_file(file.syntax()))
    }

    #[must_use]
    pub fn into_uses(self) -> BTreeMap<Name, BTreeSet<UsePath>> {
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
            types: self.types.into_iter().chain(other.types).collect(),
        }
    }

    pub(crate) fn into_graph(self) -> ItemGraph {
        let ReferencedNames { declared, implemented, stand_ins, types } = self;
        let foreign = implemented.into_iter().filter(|(owner, _)| declared.contains_key(owner)).collect();
        let named =
            Self::united(declared, foreign).into_iter().map(|(owner, uses)| (owner, Self::naming(&uses, &types)));
        ItemGraph::of_references(named.collect()).standing_in(&stand_ins)
    }

    fn naming(uses: &[UsePath], types: &BTreeSet<Name>) -> Vec<Name> {
        uses.iter().filter_map(|path| path.naming(|name| types.contains(name).then_some(name)).cloned()).collect()
    }

    fn united(all: BTreeMap<Name, Vec<UsePath>>, more: BTreeMap<Name, Vec<UsePath>>) -> BTreeMap<Name, Vec<UsePath>> {
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
            Item::Macro(call) => (&call.attrs, MacroItems::defined_by(call), Cow::Owned(MacroItems::declared_by(call))),
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
                    _ => Kind::of_item(item).declaring(owner.clone()).merged(ReferencedNames {
                        declared: Self::named(owner, scope, Spoken::of_item(item)),
                        ..ReferencedNames::default()
                    }),
                };
                self.merged(own).merged(Self::of_items(&nested, scope))
            }
        }
    }

    fn named(owner: Option<Name>, scope: &Scope<'_>, item: impl ToTokens) -> BTreeMap<Name, Vec<UsePath>> {
        owner
            .filter(|owner| *owner != Self::UNNAMED)
            .map(|owner| (owner, scope.referenced(item.into_token_stream())))
            .into_iter()
            .collect()
    }
}
