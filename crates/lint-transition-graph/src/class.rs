use newtype::Counter;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    Covered,
    Primitive,
}

impl Class {
    pub(crate) fn covered(self) -> Count {
        match self {
            Class::Covered => Count::zero().incremented(),
            Class::Primitive => Count::zero(),
        }
    }
}
