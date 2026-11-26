mod boundary;

use port_core::Op;

pub(crate) use boundary::Escaped;
pub use boundary::{Ideal, IdealId};

#[derive(Clone, Copy, Debug, Op)]
pub struct Ideals;

#[derive(Debug, Op)]
pub struct Idealised(Vec<Ideal>);

impl From<Vec<Ideal>> for Idealised {
    fn from(ideals: Vec<Ideal>) -> Idealised {
        Idealised(ideals)
    }
}
