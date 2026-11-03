use lint_layout::Name;

use super::mark::Mark;
use super::slot::Slot;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShapeToken {
    Literal,
    Local(Slot),
    Word(Name),
    Mark(Mark),
}
