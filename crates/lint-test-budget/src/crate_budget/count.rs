use std::fmt;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(usize);

impl fmt::Display for Count {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
