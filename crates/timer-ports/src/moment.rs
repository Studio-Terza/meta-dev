use std::time::Duration;

use crate::Elapsed;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment(Duration);

impl Moment {
    #[must_use]
    pub fn since(self, earlier: Moment) -> Elapsed {
        Elapsed::from(self.0.saturating_sub(earlier.0))
    }
}

impl From<Duration> for Moment {
    fn from(since_start: Duration) -> Moment {
        Moment(since_start)
    }
}
