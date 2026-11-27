use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpName(&'static str);

impl OpName {
    #[must_use]
    pub const fn new(path: &'static str) -> OpName {
        OpName(path)
    }
}

impl Display for OpName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
