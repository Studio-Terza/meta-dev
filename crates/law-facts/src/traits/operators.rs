use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct Operators;

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

    #[must_use]
    pub fn named(name: &Name) -> Option<&Name> {
        Some(name).filter(|name| name.matches::<Whole>(&Self::NAMES).is_some())
    }
}
