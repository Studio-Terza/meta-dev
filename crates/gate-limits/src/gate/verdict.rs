use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Within,
    Warn,
    Block,
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Within => formatter.write_str("ok"),
            Self::Warn => formatter.write_str("WARN"),
            Self::Block => formatter.write_str("BLOCK"),
        }
    }
}
