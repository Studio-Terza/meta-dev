use newtype::{Arbitrary, Display, Newtype};

pipeline::measured! {
    #[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Count(usize);
}
