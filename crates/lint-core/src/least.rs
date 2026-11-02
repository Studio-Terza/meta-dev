use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Least(usize);

impl Least {
    #[must_use]
    pub const fn allows(self, count: usize) -> Option<usize> {
        if count >= self.0 { Some(count) } else { None }
    }
}
