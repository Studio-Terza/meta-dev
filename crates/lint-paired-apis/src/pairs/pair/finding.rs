use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding<'method> {
    opening: &'method Name,
    closing: Name,
}

impl<'method> Finding<'method> {
    pub const fn new(opening: &'method Name, closing: Name) -> Self {
        Finding { opening, closing }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} pairs with fn {}", self.opening, self.closing)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("write a guard whose Drop does the ending")
    }
}
