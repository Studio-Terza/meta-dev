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
