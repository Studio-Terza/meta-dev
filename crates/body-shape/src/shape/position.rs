#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Position {
    Named,
    BeforeLiteral,
    Plain,
}
