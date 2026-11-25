use std::collections::BTreeSet;

use graph::{Implementation, ItemId, ItemPath, Model};
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(super) struct Stores(BTreeSet<ItemId>);

impl From<&Model> for Stores {
    fn from(model: &Model) -> Stores {
        let declared = model.items().filter(|placed| placed.krate().matches::<Whole>(&Stores::CRATES).is_some());
        let records = declared.filter_map(|placed| {
            let id = placed.entity().declared();
            let named: &Name = model.declared(id)?.as_ref();
            named.matches::<Whole>(&[Stores::RECORD]).map(|_| id)
        });
        Stores(records.collect())
    }
}

impl Stores {
    const RECORD: Word<'static> = Word::new("Record");
    const CRATES: [Word<'static>; 2] = words!["store_ports", "store_ports_macros"];
    const PATH: [Word<'static>; 2] = words!["store_ports", "Record"];

    pub(super) fn admits(&self, implementation: &Implementation) -> Option<()> {
        let resolved = implementation.implemented()?;
        let workspace = resolved.crate_item().filter(|id| self.0.contains(id)).map(|_| ());
        let path: ItemPath = Self::PATH.into_iter().collect();
        let external = resolved.external().filter(|external| *external.path() == path).map(|_| ());
        workspace.or(external)
    }

    pub(super) fn record(name: &Name) -> Option<()> {
        name.matches::<Whole>(&[Self::RECORD]).map(|_| ())
    }
}
