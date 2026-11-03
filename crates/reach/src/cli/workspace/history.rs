use std::fmt;

use super::fan_in::FanIn;
use super::reached::Reached;

#[derive(Debug)]
pub struct History<'meta> {
    reached: Reached<'meta>,
    fan_in: FanIn<'meta>,
}

impl<'meta> History<'meta> {
    pub fn new(reached: Reached<'meta>, fan_in: FanIn<'meta>) -> History<'meta> {
        History { reached, fan_in }
    }
}

impl fmt::Display for History<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "reach {}\tfan-in {}", self.reached.share(), self.fan_in.inline())
    }
}
