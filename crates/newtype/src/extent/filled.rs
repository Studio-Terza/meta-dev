use super::{Extent, Measured};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Filled<Items>(Items);

impl<Items: Measured> Filled<Items> {
    #[must_use]
    pub fn new(items: Items) -> Option<Filled<Items>> {
        match items.extent() {
            Extent::Filled => Some(Filled(items)),
            Extent::Empty => None,
        }
    }
}

impl<Items> Filled<Items> {
    #[must_use]
    pub fn into_inner(self) -> Items {
        self.0
    }
}

impl<Item> Filled<Vec<Item>> {
    pub fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.0.iter()
    }
}

impl<Item> IntoIterator for Filled<Vec<Item>> {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'filled, Item> IntoIterator for &'filled Filled<Vec<Item>> {
    type Item = &'filled Item;
    type IntoIter = std::slice::Iter<'filled, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<Item> From<Filled<Vec<Item>>> for Vec<Item> {
    fn from(filled: Filled<Vec<Item>>) -> Vec<Item> {
        filled.0
    }
}
