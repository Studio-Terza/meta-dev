use super::OpName;

impl OpName {
    #[must_use]
    pub const fn new(path: &'static str) -> OpName {
        OpName(path)
    }
}
