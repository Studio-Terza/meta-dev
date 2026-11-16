use syn::{Item, ItemMod};

#[derive(Clone, Debug)]
pub struct Nested<'items> {
    module: &'items ItemMod,
    ancestors: Vec<&'items ItemMod>,
}

impl<'items> Nested<'items> {
    #[must_use]
    pub fn modules(items: &'items [Item]) -> Vec<Nested<'items>> {
        Self::within(items, &[])
    }

    #[must_use]
    pub fn module(&self) -> &'items ItemMod {
        self.module
    }

    #[must_use]
    pub fn ancestors(&self) -> &[&'items ItemMod] {
        &self.ancestors
    }

    fn within(items: &'items [Item], ancestors: &[&'items ItemMod]) -> Vec<Nested<'items>> {
        items.iter().flat_map(|item| Self::of_item(item, ancestors)).collect()
    }

    fn of_module(module: &'items ItemMod, ancestors: &[&'items ItemMod]) -> Vec<Nested<'items>> {
        let here = Nested { module, ancestors: ancestors.to_vec() };
        let inner: Vec<&ItemMod> = ancestors.iter().copied().chain([module]).collect();
        let below = module.content.iter().flat_map(|(_, content)| Self::within(content, &inner));
        [here].into_iter().chain(below).collect()
    }

    fn of_item(item: &'items Item, ancestors: &[&'items ItemMod]) -> Vec<Nested<'items>> {
        match item {
            Item::Mod(module) => Self::of_module(module, ancestors),
            _ => Vec::new(),
        }
    }
}
