use proc_macro2::{Ident, TokenTree};
use syn::spanned::Spanned;
use syn::{Attribute, Field, ImplItem, Item, MetaList, Visibility};

use lint_core::Walk;
use lint_layout::LineNumber;

use crate::finding::{Finding, Gate};
use crate::flagged::Flagged;

#[derive(Debug)]
pub(super) struct ItemHead<'item> {
    attributes: &'item [Attribute],
    visibility: &'item Visibility,
}

impl<'item> ItemHead<'item> {
    const DOCUMENTATION: &str = "doc";
    const HIDDEN: &str = "hidden";

    pub(super) fn of_item(item: &'item Item) -> Option<ItemHead<'item>> {
        match item {
            Item::Const(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Enum(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Fn(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Mod(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Static(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Struct(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Trait(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Type(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Union(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            Item::Use(item) => Some(ItemHead { attributes: &item.attrs, visibility: &item.vis }),
            _ => None,
        }
    }

    pub(super) fn of_member(member: &'item ImplItem) -> Option<ItemHead<'item>> {
        match member {
            ImplItem::Const(member) => Some(ItemHead { attributes: &member.attrs, visibility: &member.vis }),
            ImplItem::Fn(member) => Some(ItemHead { attributes: &member.attrs, visibility: &member.vis }),
            ImplItem::Type(member) => Some(ItemHead { attributes: &member.attrs, visibility: &member.vis }),
            _ => None,
        }
    }

    pub(super) fn of_field(field: &'item Field) -> ItemHead<'item> {
        ItemHead { attributes: &field.attrs, visibility: &field.vis }
    }

    pub(super) fn flagged(&self, enclosing: Option<Gate>) -> Option<Flagged> {
        Self::gate_of(self.attributes)
            .or(enclosing)
            .filter(|_| matches!(self.visibility, Visibility::Public(_) | Visibility::Restricted(_)))
            .map(|gate| Flagged::new(LineNumber::from(self.visibility.span()), Finding::from(gate)))
    }

    pub(super) fn gate_of(attributes: &[Attribute]) -> Option<Gate> {
        (Walk::outside_tests(attributes) == Walk::Skipped)
            .then_some(Gate::GatedOnTest)
            .or_else(|| attributes.iter().find_map(Self::hiding).map(|_| Gate::DocHidden))
    }

    fn hiding(attribute: &Attribute) -> Option<&Attribute> {
        attribute
            .meta
            .require_list()
            .ok()
            .filter(|list| list.path.is_ident(Self::DOCUMENTATION))
            .and_then(Self::hidden_named)
            .map(|_| attribute)
    }

    fn hidden_named(list: &MetaList) -> Option<Ident> {
        list.tokens.clone().into_iter().filter_map(Self::ident).find(|ident| *ident == Self::HIDDEN)
    }

    fn ident(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident),
            _ => None,
        }
    }
}
