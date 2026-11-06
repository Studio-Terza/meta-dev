use design_model::{Ordinal, Prefix};
use grammar::directive::Cited;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentRef {
    prefix: Prefix,
    ordinal: Ordinal,
}

impl SegmentRef {
    #[must_use]
    pub fn new(prefix: Prefix, ordinal: Ordinal) -> SegmentRef {
        SegmentRef { prefix, ordinal }
    }
}

impl From<&Cited> for SegmentRef {
    fn from(cited: &Cited) -> SegmentRef {
        SegmentRef::new(Prefix::new(cited.prefix().to_owned()), Ordinal::new(cited.ordinal()))
    }
}

impl AsRef<Prefix> for SegmentRef {
    fn as_ref(&self) -> &Prefix {
        &self.prefix
    }
}

impl From<&SegmentRef> for Ordinal {
    fn from(reference: &SegmentRef) -> Ordinal {
        reference.ordinal
    }
}
