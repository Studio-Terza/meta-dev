use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scale(usize);

impl Scale {
    pub const ALL: [Scale; 3] = [Scale(1), Scale(2), Scale(4)];
}
