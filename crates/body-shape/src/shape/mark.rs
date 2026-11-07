use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Mark(char);

impl Mark {
    pub(crate) const DOT: Self = Self::new('.');
    pub(crate) const COLON: Self = Self::new(':');
    pub(crate) const BANG: Self = Self::new('!');
    pub(crate) const MINUS: Self = Self::new('-');
    pub(crate) const OPEN_PAREN: Self = Self::new('(');
    pub(crate) const CLOSE_PAREN: Self = Self::new(')');
    pub(crate) const OPEN_BRACE: Self = Self::new('{');
    pub(crate) const CLOSE_BRACE: Self = Self::new('}');
    pub(crate) const OPEN_BRACKET: Self = Self::new('[');
    pub(crate) const CLOSE_BRACKET: Self = Self::new(']');
}
