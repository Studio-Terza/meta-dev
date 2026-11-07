use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Context,
    Recorded,
    Actual,
}

impl fmt::Display for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context => formatter.write_str(" "),
            Self::Recorded => formatter.write_str("-"),
            Self::Actual => formatter.write_str("+"),
        }
    }
}
