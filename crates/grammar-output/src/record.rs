use crate::boundary::{Fields, Source};

#[derive(Debug)]
pub struct Parsed<T, Rest> {
    value: T,
    rest: Rest,
}

pub trait Record<From: Source = Fields>: Sized {
    fn read(from: From) -> Result<Parsed<Self, From>, From::Unread>;
}

impl<T, Rest> Parsed<T, Rest> {
    #[must_use]
    pub fn new(value: T, rest: Rest) -> Parsed<T, Rest> {
        Parsed { value, rest }
    }

    pub fn concluded<Out>(self, into: impl FnOnce(T, Rest) -> Out) -> Out {
        into(self.value, self.rest)
    }

    pub fn map<Into>(self, convert: impl FnOnce(T) -> Into) -> Parsed<Into, Rest> {
        Parsed { value: convert(self.value), rest: self.rest }
    }

    pub fn converted<Into: From<T>>(self) -> Parsed<Into, Rest> {
        self.map(Into::from)
    }
}
