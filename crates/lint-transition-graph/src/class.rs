use newtype::Counter;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    Covered,
    Primitive,
}

impl Class {
    pub(crate) fn covering(self, covered: Count) -> Count {
        match self {
            Class::Covered => covered.incremented(),
            Class::Primitive => covered,
        }
    }
}
