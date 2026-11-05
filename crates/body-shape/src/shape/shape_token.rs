use lint_layout::Name;

use super::mark::Mark;
use super::slot::Slot;
use super::spelling::Spelling;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShapeToken<'raw> {
    Literal,
    Spelled(&'raw Spelling),
    Local(Slot),
    Word(&'raw Name),
    Mark(Mark),
}
