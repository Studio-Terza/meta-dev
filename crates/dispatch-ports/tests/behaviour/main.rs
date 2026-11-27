#![deny(clippy::wildcard_enum_match_arm)]

mod ignored;
mod ports;

use std::time::Duration;

use dispatch_ports::{Moment, Pause, Wall};
use fixture::Properties;
use proptest::prelude::{any, prop_assert_eq, proptest};

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn pauses_last_whole_seconds_and_moments_count_milliseconds_either_way(
        seconds in any::<u32>(),
        early in any::<i32>(),
        late in any::<i32>(),
    ) {
        prop_assert_eq!(Duration::from(Pause::new(seconds)), Duration::from_secs(u64::from(seconds)));
        let [first, last] = [early, late].map(|second| Moment::from(Wall::new(i64::from(second))));
        let apart = i64::try_from(Duration::from_secs(u64::from(late.abs_diff(early))).as_millis())?;
        prop_assert_eq!(last.since(first), if late >= early { apart } else { -apart });
    }
}
