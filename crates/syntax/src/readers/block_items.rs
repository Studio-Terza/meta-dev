use syn::visit::Visit;
use syn::{Block, Item};

#[derive(Debug, Default)]
pub struct BlockItems<'block>(Vec<&'block Item>);

impl<'block> BlockItems<'block> {
    #[must_use]
    pub fn declared(block: &'block Block) -> Vec<&'block Item> {
        let mut found = BlockItems::default();
        found.visit_block(block);
        found.0
    }
}

impl<'block> Visit<'block> for BlockItems<'block> {
    fn visit_item(&mut self, item: &'block Item) {
        self.0.push(item);
    }
}
