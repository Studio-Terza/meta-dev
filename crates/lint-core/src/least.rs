#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Least(usize);

impl Least {
    #[must_use]
    pub const fn new(count: usize) -> Least {
        Least(count)
    }

    #[must_use]
    pub const fn allows(self, count: usize) -> bool {
        count >= self.0
    }
}
