pub trait Extended<Item>: Extend<Item> + Sized {
    #[must_use]
    fn extended(mut self, more: impl IntoIterator<Item = Item>) -> Self {
        self.extend(more);
        self
    }
}

impl<Item, Grown: Extend<Item>> Extended<Item> for Grown {}
