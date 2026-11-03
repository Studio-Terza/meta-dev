use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Context,
    Recorded,
    Actual,
}

impl fmt::Display for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Mark::Context => " ",
            Mark::Recorded => "-",
            Mark::Actual => "+",
        })
    }
}
