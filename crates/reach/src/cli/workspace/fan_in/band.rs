use std::fmt;

use newtype::Newtype;

use crate::cli::workspace::percent::Percent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band(usize);

impl Band {
    const TENTHS: usize = 10;
    const WIDTH: usize = Percent::WHOLE / Self::TENTHS;

    pub fn all() -> impl Iterator<Item = Band> {
        (0..Self::TENTHS).map(Band)
    }

    pub fn holding(percent: Percent) -> Band {
        Band(percent.into_inner() / Self::WIDTH)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lowest = self.0 * Self::WIDTH;
        write!(formatter, "{lowest}-{}%", lowest + Self::WIDTH - 1)
    }
}
