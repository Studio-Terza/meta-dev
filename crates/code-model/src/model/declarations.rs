use std::collections::BTreeMap;

use crate::boundary::ItemId;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;
use crate::resolved::Resolved;

#[derive(Debug)]
struct StdAlias {
    path: ItemPath,
    alias: Alias,
}

#[derive(Debug, Default)]
pub(crate) struct Aliases {
    declared: BTreeMap<ItemId, Alias>,
    std: Vec<StdAlias>,
}

impl Aliases {
    pub(crate) fn declare(&mut self, item: ItemId, alias: Alias) {
        self.declared.insert(item, alias);
    }

    pub(crate) fn declare_std(&mut self, path: ItemPath, alias: Alias) {
        self.std.push(StdAlias { path, alias });
    }

    pub(crate) fn aliasing(&self, path: &Resolved) -> Option<&Alias> {
        let declared = path.crate_item().and_then(|item| self.declared.get(&item));
        declared.or_else(|| {
            let std = path.std()?;
            self.std.iter().find(|row| row.path == *std).map(|row| &row.alias)
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Declarations {
    items: Vec<Item>,
    aliases: Aliases,
}

impl Declarations {
    pub(crate) fn new(items: Vec<Item>, aliases: Aliases) -> Declarations {
        Declarations { items, aliases }
    }

    pub(crate) fn items(&self) -> &[Item] {
        &self.items
    }

    pub(crate) fn aliases(&self) -> &Aliases {
        &self.aliases
    }
}
