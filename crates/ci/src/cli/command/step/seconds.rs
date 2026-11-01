use std::fmt;
use std::iter::Sum;
use std::time::Duration;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, Default, PartialEq)]
pub struct Seconds(f64);

impl From<Duration> for Seconds {
    fn from(elapsed: Duration) -> Seconds {
        Seconds(elapsed.as_secs_f64())
    }
}

impl Sum for Seconds {
    fn sum<Items: Iterator<Item = Seconds>>(items: Items) -> Seconds {
        Seconds(items.map(|seconds| seconds.0).sum::<f64>() + 0.0)
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.2}", self.0)
    }
}
