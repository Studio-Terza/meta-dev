use std::ops::RangeInclusive;

use body_shape::Bounds;
use syntax::Span;
use syntax_lexer::Place;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: Place,
    end: Place,
}

impl From<Extent> for RangeInclusive<Place> {
    fn from(extent: Extent) -> RangeInclusive<Place> {
        extent.start..=extent.end
    }
}

impl Extent {
    pub(crate) fn spanning(first: Span, last: Span) -> Extent {
        Extent { start: Place::from(first.start()), end: Place::from(last.end()) }
    }
}

impl From<Extent> for Bounds {
    fn from(extent: Extent) -> Bounds {
        Bounds::new(extent.start, extent.end)
    }
}
