use std::fmt;

use super::fan_in::FanIn;
use super::reached::Reached;

#[derive(Debug)]
pub struct History {
    reached: Reached,
    fan_in: FanIn,
}

impl History {
    pub fn new(reached: Reached, fan_in: FanIn) -> History {
        History { reached, fan_in }
    }
}

impl fmt::Display for History {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "reach {}\tfan-in {}", self.reached.share(), self.fan_in.inline())
    }
}
