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
            Mark::Context => formatter.write_str(" "),
            Mark::Recorded => formatter.write_str("-"),
            Mark::Actual => formatter.write_str("+"),
        }
    }
}
