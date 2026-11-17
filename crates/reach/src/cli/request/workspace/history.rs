use newtype::Newtype;
use std::fmt;

use super::reached::Reached;

#[derive(Newtype, Debug)]
pub struct History<'meta> {
    reached: Reached<'meta>,
}

impl fmt::Display for History<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "reach {}", self.reached.share())
    }
}
