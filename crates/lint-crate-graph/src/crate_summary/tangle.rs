use std::fmt;

use crate::crate_summary::item_count::ItemCount;
use lint_layout::Name;
use newtype::Newtype;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tangle(Vec<Name>);

impl Tangle {
    pub fn new(names: Vec<Name>) -> Tangle {
        Tangle(names)
    }

    pub fn items(&self) -> ItemCount {
        ItemCount::new(self.0.len())
    }
}

impl fmt::Display for Tangle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "cycle of {} items: {}", self.0.len(), names.join(" "))
    }
}
