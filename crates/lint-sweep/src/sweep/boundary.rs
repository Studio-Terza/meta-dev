use std::any;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member(usize);

impl Member {
    pub const FIRST: Member = Member(0);

    #[must_use]
    pub const fn new(index: usize) -> Member {
        Member(index)
    }

    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spelled(fn() -> &'static str);

impl Spelled {
    #[must_use]
    pub const fn naming<Named: ?Sized>() -> Spelled {
        Spelled(any::type_name::<Named>)
    }
}

impl fmt::Display for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}
