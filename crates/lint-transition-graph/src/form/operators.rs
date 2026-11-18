use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(crate) struct Operators;

impl Operators {
    const NAMES: [Word<'static>; 22] = words![
        "Add",
        "Sub",
        "Mul",
        "Div",
        "Rem",
        "Neg",
        "Not",
        "BitAnd",
        "BitOr",
        "BitXor",
        "Shl",
        "Shr",
        "AddAssign",
        "SubAssign",
        "MulAssign",
        "DivAssign",
        "RemAssign",
        "BitAndAssign",
        "BitOrAssign",
        "BitXorAssign",
        "ShlAssign",
        "ShrAssign"
    ];

    pub(crate) fn named(name: &Name) -> Option<Word<'static>> {
        name.matches::<Whole>(&Self::NAMES)
    }
}
