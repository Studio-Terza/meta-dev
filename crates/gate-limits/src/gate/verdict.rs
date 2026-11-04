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
            Verdict::Within => formatter.write_str("ok"),
            Verdict::Warn => formatter.write_str("WARN"),
            Verdict::Block => formatter.write_str("BLOCK"),
        }
    }
}
