use crate::{Confidence, Explicitness, SenseKind, Specificity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sense {
    kind: SenseKind,
    specificity: Specificity,
    confidence: Confidence,
    explicitness: Explicitness,
}

impl Sense {
    #[must_use]
    pub fn new(kind: SenseKind, specificity: Specificity, confidence: Confidence, explicitness: Explicitness) -> Sense {
        Sense { kind, specificity, confidence, explicitness }
    }

    #[must_use]
    pub fn kind(self) -> SenseKind {
        self.kind
    }

    #[must_use]
    pub fn specificity(self) -> Specificity {
        self.specificity
    }

    #[must_use]
    pub fn confidence(self) -> Confidence {
        self.confidence
    }

    #[must_use]
    pub fn explicitness(self) -> Explicitness {
        self.explicitness
    }
}
