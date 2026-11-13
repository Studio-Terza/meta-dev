#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Port,
    Std,
    Other,
}
