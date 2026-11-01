use std::fmt;

use super::super::count::Count;
use newtype::Counter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Density(f64);

impl Density {
    const FEWEST_NODES: usize = 2;

    pub(crate) fn between(edges: Count, nodes: Count) -> Density {
        let possible = nodes.count().saturating_mul(nodes.count().saturating_sub(1));
        let ratio = |count: usize| f64::from(u32::try_from(count).unwrap_or(u32::MAX));
        Density(if nodes.count() < Self::FEWEST_NODES { 0.0 } else { ratio(edges.count()) / ratio(possible) })
    }
}

impl fmt::Display for Density {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0)
    }
}
