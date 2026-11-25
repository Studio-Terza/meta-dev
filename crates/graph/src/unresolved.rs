#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unresolved {
    Unbound,
    PossiblyMacroDeclared,
    Unread,
}
