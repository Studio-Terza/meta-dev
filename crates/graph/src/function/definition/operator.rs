use lint_layout::Name;

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Infix {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    ShiftLeft,
    ShiftRight,
    Equal,
    Less,
    LessOrEqual,
    NotEqual,
    GreaterOrEqual,
    Greater,
}

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prefix {
    Deref,
    Not,
    Negate,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    Infix(Infix),
    Prefix(Prefix),
    Tuple,
    Array,
    Index,
    Borrow,
    Call,
    Method(Name),
    Field(Name),
}
