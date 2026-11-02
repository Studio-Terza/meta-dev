use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percent(usize);

impl Percent {
    pub const WHOLE: usize = 100;
}
