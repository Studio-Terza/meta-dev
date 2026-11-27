mod boundary;

use ports::port::Op;
use ports::transcript::Body;

pub use boundary::{Fate, Halt, StopRow, Tally};

#[derive(Debug, Op)]
pub struct Stopped(StopRow);

#[derive(Clone, Copy, Debug, Op)]
pub struct StopTrail;

#[derive(Clone, Copy, Debug, Op)]
pub struct Overruns;

#[derive(Debug, Op)]
pub struct Overran(Vec<Body>);

impl From<StopRow> for Stopped {
    fn from(row: StopRow) -> Stopped {
        Stopped(row)
    }
}

impl From<Vec<Body>> for Overran {
    fn from(tasks: Vec<Body>) -> Overran {
        Overran(tasks)
    }
}
