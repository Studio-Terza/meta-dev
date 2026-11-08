mod kind;
mod macro_items;
mod parameters;
mod referenced;
mod spoken;

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Attribute, Item, ItemImpl};

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
        more.into_iter().fold(all, |mut united, (owner, uses)| {
            united.entry(owner).or_default().extend(uses);
            united
        })
    }

    fn with_item(self, item: &Item, scope: &Scope<'_>) -> ReferencedNames {
        let own = Parts::of_item(item).entered().map(|parts| parts.referenced(scope));
        self.merged(own.unwrap_or_default())
    }

    fn named(owner: Option<Name>, scope: &Scope<'_>, parameters: &Parameters, item: impl ToTokens) -> ReferencedNames {
        owner
            .filter(|candidate| *candidate != Name::from(Self::UNNAMED))
            .map_or_else(ReferencedNames::default, |named| {
                Referenced::of_tokens(scope, item.into_token_stream()).owned(named, parameters)
            })
    }
}

#[derive(Debug)]
struct Parts<'item> {
    item: &'item Item,
    attributes: &'item [Attribute],
    owner: Option<Name>,
    nested: Cow<'item, [Item]>,
}

impl<'item> Parts<'item> {
    pub fn of_item(item: &'item Item) -> Parts<'item> {
        match item {
            Item::Struct(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Enum(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Trait(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Fn(found) => Parts::named(item, &found.attrs, &found.sig.ident),
            Item::Type(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Const(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Static(found) => Parts::named(item, &found.attrs, &found.ident),
            Item::Impl(found) => Parts { item, attributes: &found.attrs, owner: None, nested: Cow::Borrowed(&[]) },
            Item::Mod(module) => Parts {
                item,
                attributes: &module.attrs,
                owner: None,
                nested: module
                    .content
                    .as_ref()
                    .map_or(Cow::Borrowed(&[]), |(_, items)| Cow::Borrowed(items.as_slice())),
            },
            Item::Macro(call) => Parts {
                item,
                attributes: &call.attrs,
                owner: MacroItems::defined_by(call),
                nested: Cow::Owned(MacroItems::declared_by(call)),
            },
            _ => Parts { item, attributes: &[], owner: None, nested: Cow::Borrowed(&[]) },
        }
    }

    pub fn entered(self) -> Option<Parts<'item>> {
        (Walk::outside_tests(self.attributes) == Walk::Entered).then_some(self)
    }

    pub fn referenced(self, scope: &Scope<'_>) -> ReferencedNames {
        let parameters = Parameters::of_item(self.item);
        let nested = ReferencedNames::of_items(&self.nested, scope);
        self.own(scope, &parameters).merged(nested)
    }

    fn named(item: &'item Item, attributes: &'item [Attribute], ident: &Ident) -> Parts<'item> {
        Parts { item, attributes, owner: Some(Name::from(ident)), nested: Cow::Borrowed(&[]) }
    }

    fn own(self, scope: &Scope<'_>, parameters: &Parameters) -> ReferencedNames {
        match self.item {
            Item::Impl(found) => Self::implementing(found, scope, parameters),
            _ => Kind::of_item(self.item).declaring(self.owner.clone()).merged(ReferencedNames::named(
                self.owner,
                scope,
                parameters,
                Spoken::of_item(self.item),
            )),
        }
    }

    fn implementing(found: &ItemImpl, scope: &Scope<'_>, parameters: &Parameters) -> ReferencedNames {
        let read = ReferencedNames::named(Name::of_type(&found.self_ty), scope, parameters, found);
        ReferencedNames { implemented: read.declared, declared: BTreeMap::new(), ..read }
    }
}
