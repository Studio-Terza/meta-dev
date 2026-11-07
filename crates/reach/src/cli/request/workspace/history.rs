use std::fmt;

use super::reached::Reached;

#[derive(Debug)]
pub struct History<'meta> {
    reached: Reached<'meta>,
}

impl<'meta> History<'meta> {
    pub const fn new(reached: Reached<'meta>) -> Self {
        History { reached }
    }
}

impl fmt::Display for History<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "reach {}", self.reached.share())
    }
}
