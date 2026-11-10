use std::time::Duration;

use timer_ports::{Elapsed, Moment};

laws::table!(
    a_moment_since_an_earlier_one_is_the_time_between_them_and_never_negative,
    |(later, earlier): &(u64, u64)| Moment::from(Duration::from_millis(*later)).since(Moment::from(Duration::from_millis(*earlier))),
    [
        (5_000, 2_000) => Elapsed::from(Duration::from_millis(3_000)),
        (2_000, 5_000) => Elapsed::from(Duration::ZERO),
        (7, 7) => Elapsed::from(Duration::ZERO),
    ]
);
