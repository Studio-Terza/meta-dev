use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCount(usize);

impl From<usize> for EdgeCount {
    fn from(edges: usize) -> EdgeCount {
        EdgeCount(edges)
    }
}

impl fmt::Display for EdgeCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
