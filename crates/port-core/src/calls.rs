use newtype::{Arbitrary, Newtype};

pipeline::measured! {
    #[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Calls(usize);
}
