use std::cmp::Ordering;
use std::fmt;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Conductance(f64);

impl Conductance {
    const LOOSE_BELOW: f64 = 0.19;

    pub fn loose(self) -> Option<Conductance> {
        match self.0.total_cmp(&Self::LOOSE_BELOW) {
            Ordering::Less => Some(self),
            Ordering::Equal | Ordering::Greater => None,
        }
    }

    pub fn total_cmp(self, other: Conductance) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for Conductance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0.abs())
    }
}
