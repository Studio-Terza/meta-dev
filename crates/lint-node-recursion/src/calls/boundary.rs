#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Callable(usize);

impl Callable {
    pub(super) fn every<Item>(items: &[Item]) -> impl Iterator<Item = Callable> + use<Item> {
        (0..items.len()).map(Callable)
    }

    pub(super) fn picked<Item>(self, items: &[Item]) -> Option<&Item> {
        items.get(self.0)
    }
}
