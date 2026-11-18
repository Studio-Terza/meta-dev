use body_shape::Bounds;
use syntax::Span;
use syntax_lexer::Place;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: Place,
    end: Place,
}

impl Extent {
    pub(crate) fn holds(&self, inner: &Extent) -> Option<()> {
        Some(()).filter(|()| self.start <= inner.start).filter(|()| inner.end <= self.end)
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
