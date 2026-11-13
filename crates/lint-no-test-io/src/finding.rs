use std::fmt;

use graph::ItemPath;
use lint_core::Fix;

#[derive(Debug)]
pub struct Finding(ItemPath);

impl From<ItemPath> for Finding {
    fn from(path: ItemPath) -> Finding {
        Finding(path)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "calls {} in test code", self.0)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("use a fake of the port or a library API such as golden!")
    }
}
