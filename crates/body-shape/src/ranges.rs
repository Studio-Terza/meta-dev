mod boundary;

use syntax::Place;

pub use boundary::Shaper;

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
