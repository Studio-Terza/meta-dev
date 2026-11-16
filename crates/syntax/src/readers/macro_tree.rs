use syn::{File, Item, Macro};

#[derive(Debug)]
pub struct MacroTree;

impl MacroTree {
    pub fn preorder(root: &Macro, nested: &impl Fn(&Macro) -> Vec<Macro>) -> Vec<Macro> {
        let below = nested(root).iter().flat_map(|inner| Self::preorder(inner, nested)).collect::<Vec<Macro>>();
        [root.clone()].into_iter().chain(below).collect()
    }

    pub fn frontier(
        root: &Macro,
        nested: &impl Fn(&Macro) -> Vec<Macro>,
        stops: &impl Fn(&Macro) -> Option<()>,
    ) -> Vec<Macro> {
        stops(root).map_or_else(
            || nested(root).iter().flat_map(|inner| Self::frontier(inner, nested, stops)).collect(),
            |()| vec![root.clone()],
        )
    }

    #[must_use]
    pub fn expanded_items(items: &[Item]) -> Vec<Item> {
        let invoked = items.iter().filter_map(Self::item_macro);
        let expanded = invoked.flat_map(|invocation| {
            syn::parse2::<File>(invocation.tokens.clone()).map_or_else(|_| Vec::new(), |file| file.items)
        });
        expanded.flat_map(|item| [item.clone()].into_iter().chain(Self::expanded_items(&[item]))).collect()
    }

    fn item_macro(item: &Item) -> Option<&Macro> {
        match item {
            Item::Macro(found) => found.ident.as_ref().map_or(Some(&found.mac), |_| None),
            _ => None,
        }
    }
}
