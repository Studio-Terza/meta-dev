#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    Written,
    Unchanged,
    Planned,
}
