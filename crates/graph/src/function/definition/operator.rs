use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prefix {
    Deref,
    Not,
    Negate,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
