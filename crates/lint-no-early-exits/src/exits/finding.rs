use lint_core::Fix;
use strum::Display;

#[derive(Display, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Finding {
    Return,
    Break,
    Continue,
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "yield the value from an if or match expression, drive the iteration with find, take_while, try_fold or position, or split it into smaller functions",
        )
    }
}
