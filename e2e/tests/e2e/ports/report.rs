use std::fmt;
use std::io;

use e2e::ports::Measurements;

#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    part: usize,
    whole: usize,
}

impl Fraction {
    const PLACES: usize = 1_000;

    pub fn of(part: usize, whole: usize) -> Fraction {
        Fraction { part, whole }
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scaled = (self.part * Self::PLACES).checked_div(self.whole).unwrap_or_default();
        write!(formatter, "{}.{:03}", scaled / Self::PLACES, scaled % Self::PLACES)
    }
}

#[derive(Debug)]
pub struct Report {
    scenario: &'static str,
}

impl Report {
    pub fn named(scenario: &'static str) -> Report {
        Report { scenario }
    }

    pub fn line(&self, name: &str, value: impl fmt::Display) -> io::Result<()> {
        Measurements::record(&format!("e2e {} {name}={value}\n", self.scenario))
    }
}
