use syn::visit;
use syn::{Block, ItemMod};

use crate::namespace::{Entries, ScopeKind};

use super::{Lowering, Slot};

impl Lowering<'_, '_> {
    pub(super) fn module_entered(&mut self, item: &ItemMod) {
        match &item.content {
            Some((_, items)) => self.module_visited(item, items),
            None => self.slots.push(Slot::new(&self.here, self.context.region(), item)),
        }
    }

    pub(super) fn block_scoped(&mut self, block: &Block) {
        let outer = self.entered(ScopeKind::Block, Entries::of_block(&block.stmts), self.here.children());
        visit::visit_block(self, block);
        self.here = outer;
    }
}
