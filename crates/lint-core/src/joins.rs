pub trait Joins: Default {
    #[must_use]
    fn combine(self, later: Self) -> Self;

    #[must_use]
    fn concat(parts: impl IntoIterator<Item = Self>) -> Self {
        parts.into_iter().fold(Self::default(), Self::combine)
    }
}

impl Joins for () {
    fn combine(self, (): ()) {}
}

impl<Item> Joins for Vec<Item> {
    fn combine(mut self, later: Vec<Item>) -> Vec<Item> {
        self.extend(later);
        self
    }
}
