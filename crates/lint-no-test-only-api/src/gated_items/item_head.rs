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
            Item::Const(constant) => Some(ItemHead { attributes: &constant.attrs, visibility: &constant.vis }),
            Item::Enum(enumeration) => Some(ItemHead { attributes: &enumeration.attrs, visibility: &enumeration.vis }),
            Item::Fn(function) => Some(ItemHead { attributes: &function.attrs, visibility: &function.vis }),
            Item::Mod(module) => Some(ItemHead { attributes: &module.attrs, visibility: &module.vis }),
            Item::Static(global) => Some(ItemHead { attributes: &global.attrs, visibility: &global.vis }),
            Item::Struct(structure) => Some(ItemHead { attributes: &structure.attrs, visibility: &structure.vis }),
            Item::Trait(trait_item) => Some(ItemHead { attributes: &trait_item.attrs, visibility: &trait_item.vis }),
            Item::Type(alias) => Some(ItemHead { attributes: &alias.attrs, visibility: &alias.vis }),
            Item::Union(union) => Some(ItemHead { attributes: &union.attrs, visibility: &union.vis }),
            Item::Use(import) => Some(ItemHead { attributes: &import.attrs, visibility: &import.vis }),
            _ => None,
        }
    }

    pub(super) fn of_member(member: &'item ImplItem) -> Option<ItemHead<'item>> {
        match member {
            ImplItem::Const(constant) => Some(ItemHead { attributes: &constant.attrs, visibility: &constant.vis }),
            ImplItem::Fn(function) => Some(ItemHead { attributes: &function.attrs, visibility: &function.vis }),
            ImplItem::Type(alias) => Some(ItemHead { attributes: &alias.attrs, visibility: &alias.vis }),
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
