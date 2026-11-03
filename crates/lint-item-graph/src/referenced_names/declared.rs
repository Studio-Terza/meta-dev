mod macro_items;

use std::borrow::Cow;

use syn::{Attribute, Item};

use lint_core::{Syntax, Walk};
use lint_layout::Name;

use super::ReferencedNames;
use crate::scope::Scope;
use macro_items::MacroItems;

#[derive(Debug)]
pub struct Declared<'item> {
    item: &'item Item,
    attributes: &'item [Attribute],
    owner: Option<Name>,
    nested: Cow<'item, [Item]>,
}

impl<'item> Declared<'item> {
    #[must_use]
    pub fn entered(self) -> Option<Declared<'item>> {
        (Walk::outside_tests(self.attributes) == Walk::Entered).then_some(self)
    }

    #[must_use]
    pub fn referenced(self, scope: &Scope<'_>) -> ReferencedNames {
        let Declared { item, owner, nested, .. } = self;
        Self::own(item, owner, scope).merged(ReferencedNames::of_items(&nested, scope))
    }

    fn new(
        item: &'item Item,
        attributes: &'item [Attribute],
        owner: Option<Name>,
        nested: Cow<'item, [Item]>,
    ) -> Declared<'item> {
        Declared { item, attributes, owner, nested }
    }

    fn own(item: &Item, owner: Option<Name>, scope: &Scope<'_>) -> ReferencedNames {
        match item {
            Item::Impl(found) => ReferencedNames {
                implemented: ReferencedNames::named(Syntax::last_name(&found.self_ty), scope, found),
                ..ReferencedNames::default()
            },
            _ => ReferencedNames { declared: ReferencedNames::named(owner, scope, item), ..ReferencedNames::default() },
        }
    }
}

impl<'item> From<&'item Item> for Declared<'item> {
    fn from(item: &'item Item) -> Declared<'item> {
        match item {
            Item::Struct(found) => {
                Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[]))
            }
            Item::Enum(found) => Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[])),
            Item::Trait(found) => Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[])),
            Item::Fn(found) => {
                Declared::new(item, &found.attrs, Some(Name::from(&found.sig.ident)), Cow::Borrowed(&[]))
            }
            Item::Type(found) => Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[])),
            Item::Const(found) => Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[])),
            Item::Static(found) => {
                Declared::new(item, &found.attrs, Some(Name::from(&found.ident)), Cow::Borrowed(&[]))
            }
            Item::Impl(found) => Declared::new(item, &found.attrs, None, Cow::Borrowed(&[])),
            Item::Mod(module) => Declared::new(
                item,
                &module.attrs,
                None,
                module.content.as_ref().map_or(Cow::Borrowed(&[]), |(_, items)| Cow::Borrowed(items.as_slice())),
            ),
            Item::Macro(call) => Declared::new(item, &call.attrs, None, Cow::Owned(MacroItems::declared_by(call))),
            _ => Declared::new(item, &[], None, Cow::Borrowed(&[])),
        }
    }
}
