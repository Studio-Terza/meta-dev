use crate::line_hash::LineHash;
use crate::set_aside::UnverifiedEscape;

/// One line of the side file as written, and the escape it holds when it can be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideLine {
    raw: Vec<u8>,
    escape: Option<UnverifiedEscape>,
}

impl SideLine {
    pub fn new(raw: Vec<u8>, escape: Option<UnverifiedEscape>) -> Self {
        Self { raw, escape }
    }

    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    pub fn escape(&self) -> Option<&UnverifiedEscape> {
        self.escape.as_ref()
    }

    pub(crate) fn hash(&self) -> LineHash {
        LineHash::of(&self.raw)
    }
}
