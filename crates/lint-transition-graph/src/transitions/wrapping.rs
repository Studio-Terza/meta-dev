use algebra::Semigroup;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Wrapping {
    Plain,
    Optional,
    Fallible,
    Many,
}

impl Semigroup for Wrapping {
    fn combine(self, inner: Wrapping) -> Wrapping {
        Some(inner).filter(|candidate| *candidate == Wrapping::Many).unwrap_or(self)
    }
}
