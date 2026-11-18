pub trait Semigroup: Sized {
    #[must_use]
    fn combine(self, other: Self) -> Self;
}

pub trait Monoid: Semigroup + Default {
    fn concat(parts: impl IntoIterator<Item = Self>) -> Self {
        parts.into_iter().fold(Self::default(), Self::combine)
    }
}

impl Semigroup for () {
    fn combine(self, (): ()) {}
}

impl Monoid for () {}

impl<Item> Semigroup for Vec<Item> {
    fn combine(mut self, other: Vec<Item>) -> Vec<Item> {
        self.extend(other);
        self
    }
}

impl<Item> Monoid for Vec<Item> {}
