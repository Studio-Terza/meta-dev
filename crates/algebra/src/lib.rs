pub trait Semigroup: Sized {
    #[must_use]
    fn combine(self, other: Self) -> Self;
}

pub trait Monoid: Semigroup {
    fn identity() -> Self;

    fn concat(parts: impl IntoIterator<Item = Self>) -> Self {
        parts.into_iter().fold(Self::identity(), Self::combine)
    }
}
