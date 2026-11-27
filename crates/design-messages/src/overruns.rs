mod boundary;

use port_core::Op;
use transcript_ports::Body;

#[derive(Clone, Copy, Debug, Op)]
pub struct Overruns;

#[derive(Debug, Op)]
pub struct Overran(Vec<Body>);

impl From<Vec<Body>> for Overran {
    fn from(tasks: Vec<Body>) -> Overran {
        Overran(tasks)
    }
}
