mod kind;
mod macro_items;
mod parameters;
mod referenced;
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
use parameters::Parameters;
use referenced::Referenced;
use spoken::Spoken;

#[derive(Clone, Debug, Default)]
pub struct ReferencedNames {
    declared: BTreeMap<Name, Vec<UsePath>>,
    implemented: BTreeMap<Name, Vec<UsePath>>,
    across: BTreeMap<Name, Vec<UsePath>>,
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
    pub fn uses(&self) -> BTreeMap<&Name, BTreeSet<&UsePath>> {
        self.across.iter().map(|(owner, uses)| (owner, uses.iter().collect())).collect()
    }

    pub(crate) fn of_items(items: &[Item], scope: &Scope<'_>) -> ReferencedNames {
        items.iter().fold(ReferencedNames::default(), |names, item| names.with_item(item, scope))
    }

    #[must_use]
    pub fn merged(mut self, other: ReferencedNames) -> ReferencedNames {
        self.stand_ins.extend(other.stand_ins);
        self.types.extend(other.types);
        ReferencedNames {
            declared: Self::united(self.declared, other.declared),
            implemented: Self::united(self.implemented, other.implemented),
            across: Self::united(self.across, other.across),
            ..self
        }
    }

    #[must_use]
    pub fn into_graph(self) -> ItemGraph {
        let ReferencedNames { declared, implemented, stand_ins, types, .. } = self;
        let foreign = implemented.into_iter().filter(|(owner, _)| declared.contains_key(owner)).collect();
        let named =
            Self::united(declared, foreign).into_iter().map(|(owner, uses)| (owner, Self::naming(&uses, &types)));
        ItemGraph::of_references(named.collect()).standing_in(&stand_ins)
    }

    fn naming(uses: &[UsePath], types: &BTreeSet<Name>) -> Vec<Name> {
        uses.iter()
            .filter_map(|path| path.naming(|name| types.contains(name).then_some(name)).segments().last().cloned())
            .collect()
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
                let parameters = &Parameters::of_item(item);
                let own = match item {
                    Item::Impl(item) => {
                        let read = Self::named(Name::of_type(&item.self_ty), scope, parameters, item);
                        ReferencedNames { implemented: read.declared, declared: BTreeMap::new(), ..read }
                    }
                    _ => Kind::of_item(item).declaring(owner.clone()).merged(Self::named(
                        owner,
                        scope,
                        parameters,
                        Spoken::of_item(item),
                    )),
                };
                self.merged(own).merged(Self::of_items(&nested, scope))
            }
        }
    }

    fn named(owner: Option<Name>, scope: &Scope<'_>, parameters: &Parameters, item: impl ToTokens) -> ReferencedNames {
        owner.filter(|owner| *owner != Self::UNNAMED).map_or_else(ReferencedNames::default, |owner| {
            Referenced::of_tokens(scope, item.into_token_stream()).owned(owner, parameters)
        })
    }
}
