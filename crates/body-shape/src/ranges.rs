mod boundary;

use syntax::{Block, Place};

pub use boundary::{Erasure, Shaper};

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    start: Place,
    end: Place,
}

impl Bounds {
    #[must_use]
    pub fn new(start: Place, end: Place) -> Bounds {
        Bounds { start, end }
    }
}

impl From<&Block> for Bounds {
    fn from(block: &Block) -> Bounds {
        let braces = block.brace_token.span;
        Bounds::new(Place::from(braces.open().start()), Place::from(braces.close().end()))
    }
}
