use std::fmt;
use std::ops::Range;

use newtype::proptest::collection::vec;
use newtype::proptest::sample::select;
use newtype::proptest::strategy::Strategy;
use newtype::{Arbitrary, Newtype, Wrapping};

use super::Flagged;

#[derive(Newtype, Arbitrary, Debug, Default, PartialEq, Eq)]
#[newtype(generate = Lines::sampled)]
pub struct Lines(Vec<String>);

impl Lines {
    const SAMPLED: [&'static str; 3] = ["a.rs:1", "a.rs:2", "src/b.rs:10"];
    const COUNT: Range<usize> = 0..5;

    fn sampled() -> impl Strategy<Value = Lines> {
        Strategy::prop_map(vec(select(&Self::SAMPLED[..]), Self::COUNT), |lines| {
            Lines(lines.into_iter().map(str::to_owned).collect())
        })
    }
}

impl From<Flagged> for Lines {
    fn from(flagged: Flagged) -> Lines {
        Lines(flagged.into_inner().iter().map(ToString::to_string).collect())
    }
}

impl<const WANTED: usize> From<[&str; WANTED]> for Lines {
    fn from(wanted: [&str; WANTED]) -> Lines {
        Lines(wanted.iter().map(ToString::to_string).collect())
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}
