mod boundary;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Quote {
    Outside,
    Single,
    Double,
}
