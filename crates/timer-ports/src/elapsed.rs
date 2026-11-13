use std::fmt;
use std::ops::Add;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elapsed(Duration);

impl From<Duration> for Elapsed {
    fn from(span: Duration) -> Elapsed {
        Elapsed(span)
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let milliseconds = self.0.as_millis().to_string();
        formatter.pad(&milliseconds)
    }
}

impl Add for Elapsed {
    type Output = Elapsed;

    fn add(self, later: Elapsed) -> Elapsed {
        Elapsed(self.0 + later.0)
    }
}
