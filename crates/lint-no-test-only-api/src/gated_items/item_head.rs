use proc_macro2::TokenTree;
use syn::spanned::Spanned;
use syn::{Attribute, Field, ImplItem, Item, Meta, MetaList, Visibility};

use lint_core::Walk;
use lint_layout::LineNumber;

use crate::finding::Finding;
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

    pub(super) fn flagged(&self, enclosing: Option<Finding>) -> Option<Flagged> {
        Self::gate_of(self.attributes)
            .or(enclosing)
            .filter(|_| !matches!(self.visibility, Visibility::Inherited))
            .map(|finding| Flagged::new(LineNumber::from(self.visibility.span()), finding))
    }

    pub(super) fn gate_of(attributes: &[Attribute]) -> Option<Finding> {
        (Walk::outside_tests(attributes) == Walk::Skipped)
            .then_some(Finding::GatedOnTest)
            .or_else(|| attributes.iter().any(Self::is_hidden).then_some(Finding::DocHidden))
    }

    fn is_hidden(attribute: &Attribute) -> bool {
        match &attribute.meta {
            Meta::List(list) if list.path.is_ident(Self::DOCUMENTATION) => Self::names_hidden(list),
            _ => false,
        }
    }

    fn names_hidden(list: &MetaList) -> bool {
        list.tokens.clone().into_iter().any(|tree| matches!(tree, TokenTree::Ident(ident) if ident == Self::HIDDEN))
    }
}
