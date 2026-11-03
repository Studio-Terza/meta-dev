#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pipeline,
    Router,
    Neither,
}
