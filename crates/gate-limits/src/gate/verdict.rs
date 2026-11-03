use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Within,
    Warn,
    Block,
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Verdict::Within => "ok",
            Verdict::Warn => "WARN",
            Verdict::Block => "BLOCK",
        })
    }
}
