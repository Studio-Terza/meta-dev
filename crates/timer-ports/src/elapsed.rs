use std::fmt;
use std::time::Duration;

pipeline::measured! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Elapsed(Duration);
}

impl From<Duration> for Elapsed {
    fn from(span: Duration) -> Elapsed {
        Elapsed(span)
    }
}

impl From<Elapsed> for Duration {
    fn from(elapsed: Elapsed) -> Duration {
        elapsed.0
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let milliseconds = self.0.as_millis().to_string();
        formatter.pad(&milliseconds)
    }
}
