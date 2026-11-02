use syn::visit::Visit;
use syn::{Item, ItemFn};

#[derive(Debug, Default)]
pub(crate) struct BodyItems<'ast> {
    found: Vec<&'ast Item>,
}

impl<'ast> BodyItems<'ast> {
    pub(crate) fn declared_in(main: &'ast ItemFn) -> Vec<&'ast Item> {
        let mut items = BodyItems::default();
        items.visit_block(&main.block);
        items.found
    }
}

impl<'ast> Visit<'ast> for BodyItems<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Use(_) => {}
            declared => self.found.push(declared),
        }
    }
}
