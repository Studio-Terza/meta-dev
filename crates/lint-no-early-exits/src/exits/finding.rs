use std::fmt;

use lint_core::Fix;

#[derive(Clone, Copy, Debug)]
pub enum Finding {
    Return,
    Break,
    Continue,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return => formatter.write_str("return"),
            Self::Break => formatter.write_str("break"),
            Self::Continue => formatter.write_str("continue"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "yield the value from an if or match expression, drive the iteration with find, take_while, try_fold or position, or split it into smaller functions",
        )
    }
}
