use super::{Exclusion, Inside, Relation, Shape};
use crate::boundary::Spelling;
use crate::claim::ranges::Ranges;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Single<'accepted> {
    Any,
    Ranges(&'accepted Ranges),
    Exclusion(&'accepted Exclusion),
    Shape(&'accepted Shape),
    Relation(&'accepted Relation),
    Written(&'accepted Spelling),
}

impl Single<'_> {
    pub(crate) fn in_ranges(self, outer: &Ranges) -> Option<Inside> {
        match self {
            Single::Ranges(inner) => outer.encloses(inner).map(|_| Inside),
            Single::Any | Single::Exclusion(_) | Single::Shape(_) | Single::Relation(_) | Single::Written(_) => None,
        }
    }

    pub(crate) fn avoiding(self, outer: &Exclusion) -> Option<Inside> {
        match self {
            Single::Ranges(inner) => outer.missing_from(inner),
            Single::Exclusion(inner) => outer.excluded_by(inner),
            Single::Any | Single::Shape(_) | Single::Relation(_) | Single::Written(_) => None,
        }
    }

    pub(crate) fn in_shape(self, outer: &Shape) -> Option<Inside> {
        match self {
            Single::Shape(inner) => outer.encloses(inner),
            Single::Any | Single::Ranges(_) | Single::Exclusion(_) | Single::Relation(_) | Single::Written(_) => None,
        }
    }

    pub(crate) fn in_relation(self, outer: &Relation) -> Option<Inside> {
        match self {
            Single::Relation(inner) => outer.implied_by(inner),
            Single::Any | Single::Ranges(_) | Single::Exclusion(_) | Single::Shape(_) | Single::Written(_) => None,
        }
    }

    pub(crate) fn written_as(self, outer: &Spelling) -> Option<Inside> {
        match self {
            Single::Written(inner) => (inner == outer).then_some(Inside),
            Single::Any | Single::Ranges(_) | Single::Exclusion(_) | Single::Shape(_) | Single::Relation(_) => None,
        }
    }
}
