use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Conductance(f64);

impl Conductance {
    const LOOSE_BELOW: f64 = 0.19;

    pub fn of_ratio(crossing: f64, volume: f64) -> Conductance {
        Conductance(if volume > 0.0 { crossing / volume } else { 0.0 })
    }

    pub fn is_loose(self) -> bool {
        self.0 < Self::LOOSE_BELOW
    }

    pub fn total_cmp(self, other: Conductance) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for Conductance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0.abs())
    }
}
