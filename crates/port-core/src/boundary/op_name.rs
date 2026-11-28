#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpName(&'static str);

impl OpName {
    #[must_use]
    pub const fn new(path: &'static str) -> OpName {
        OpName(path)
    }
}
