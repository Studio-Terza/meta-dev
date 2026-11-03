use std::ops::Add;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(usize);

impl Add for Score {
    type Output = Score;

    fn add(self, other: Score) -> Score {
        Score(self.0 + other.0)
    }
}
