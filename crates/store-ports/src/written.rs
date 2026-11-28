use newtype::{Arbitrary, Newtype};

pipeline::measured! {
    #[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[must_use = "a write receipt says how many rows the store changed; use it or do not ask for it"]
    pub struct Written(usize);
}
