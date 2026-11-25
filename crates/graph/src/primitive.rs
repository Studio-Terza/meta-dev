#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Primitive {
    Bool,
    Char,
    Str,
    Integer,
    Float,
}
