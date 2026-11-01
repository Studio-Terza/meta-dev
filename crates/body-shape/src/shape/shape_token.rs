use lint_layout::Name;

use super::slot::Slot;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShapeToken {
    Literal,
    Local(Slot),
    Word(Name),
    Mark(char),
}
