use body_shape::Bounds;
use syntax::{Place, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: Place,
    end: Place,
}

impl Extent {
    pub(crate) fn holds<'inner>(&self, inner: &'inner Extent) -> Option<&'inner Extent> {
        Some(inner)
            .filter(|candidate| self.start <= candidate.start)
            .filter(|starting_inside| starting_inside.end <= self.end)
    }

    pub(crate) fn spanning(first: Span, last: Span) -> Extent {
        Extent { start: Place::from(first.start()), end: Place::from(last.end()) }
    }
}

impl From<Extent> for Bounds {
    fn from(extent: Extent) -> Bounds {
        Bounds::new(extent.start, extent.end)
    }
}
