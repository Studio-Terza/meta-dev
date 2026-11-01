use syn::{ImplItem, ItemImpl, ItemTrait, Signature, TraitItem, Visibility};

#[derive(Debug)]
pub struct Members;

impl Members {
    pub fn public_in(block: &ItemImpl) -> Vec<&Signature> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) if matches!(method.vis, Visibility::Public(_)) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }

    pub fn declared_in(block: &ItemTrait) -> Vec<&Signature> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }
}
