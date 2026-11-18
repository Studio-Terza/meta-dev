use body_shape::Bounds;
use syntax::Span;
use syntax_lexer::Place;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: Place,
    end: Place,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cover {
    Covers,
    Misses,
}

impl Extent {
    pub(crate) fn cover(&self, inner: &Extent) -> Cover {
        (self.start <= inner.start).then_some(Cover::Covers).filter(|_| inner.end <= self.end).unwrap_or(Cover::Misses)
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
