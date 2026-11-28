use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct At(usize);

#[derive(Debug)]
pub(crate) struct Indexed<'graph, Item> {
    at: At,
    item: &'graph Item,
}

#[derive(Debug)]
pub(crate) struct Numbered<Item>(Vec<Item>);

impl<Item> Clone for Indexed<'_, Item> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Item> Copy for Indexed<'_, Item> {}

impl<Item> Default for Numbered<Item> {
    fn default() -> Self {
        Numbered(Vec::new())
    }
}

impl<'graph, Item> Indexed<'graph, Item> {
    pub(super) fn at(self) -> At {
        self.at
    }

    pub(super) fn item(self) -> &'graph Item {
        self.item
    }
}

impl<Item> Numbered<Item> {
    pub(super) fn push(&mut self, item: Item) {
        self.0.push(item);
    }

    pub(super) fn indexed(&self) -> impl Iterator<Item = Indexed<'_, Item>> {
        self.0.iter().enumerate().map(|(at, item)| Indexed { at: At(at), item })
    }

    pub(super) fn at(&self, at: At) -> Option<&Item> {
        self.0.get(at.0)
    }

    pub(super) fn dead(&self, alive: &BTreeSet<At>) -> impl Iterator<Item = &Item> {
        self.indexed().filter(|indexed| !alive.contains(&indexed.at)).map(Indexed::item)
    }
}
