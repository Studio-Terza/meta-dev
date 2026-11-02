use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding {
    opening: Name,
    closing: Name,
}

impl Finding {
    pub fn new(opening: Name, closing: Name) -> Finding {
        Finding { opening, closing }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} pairs with fn {}", self.opening, self.closing)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("write a guard whose Drop does the ending")
    }
}
