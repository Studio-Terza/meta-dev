use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
pub struct Seconds(f64);

impl fmt::Display for Seconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s", self.0)
    }
}
