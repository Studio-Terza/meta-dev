mod boundary;

use port_core::Op;

pub use boundary::{StopRow, Tally};

#[derive(Debug, Op)]
pub struct Stopped(StopRow);

#[derive(Clone, Copy, Debug, Op)]
pub struct StopTrail;

impl From<StopRow> for Stopped {
    fn from(row: StopRow) -> Stopped {
        Stopped(row)
    }
}
