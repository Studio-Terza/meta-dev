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
