use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Mark(char);

impl Mark {
    pub(crate) const DOT: Mark = Mark::new('.');
    pub(crate) const COLON: Mark = Mark::new(':');
    pub(crate) const BANG: Mark = Mark::new('!');
    pub(crate) const MINUS: Mark = Mark::new('-');
    pub(crate) const OPEN_PAREN: Mark = Mark::new('(');
    pub(crate) const CLOSE_PAREN: Mark = Mark::new(')');
    pub(crate) const OPEN_BRACE: Mark = Mark::new('{');
    pub(crate) const CLOSE_BRACE: Mark = Mark::new('}');
    pub(crate) const OPEN_BRACKET: Mark = Mark::new('[');
    pub(crate) const CLOSE_BRACKET: Mark = Mark::new(']');
}
