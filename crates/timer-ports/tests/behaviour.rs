use std::time::Duration;

use proptest::prelude::{prop_assert_eq, proptest};
use timer_ports::{Elapsed, Moment, Monotonic, Timer};

const PAUSE: Duration = Duration::from_millis(5);
const MOST: u64 = u64::MAX / 4;

proptest! {
    #[test]
    fn a_moment_since_an_earlier_one_is_the_time_between_them_and_never_negative(later in 0..MOST, earlier in 0..MOST) {
        let since = Moment::from(Duration::from_millis(later)).since(Moment::from(Duration::from_millis(earlier)));
        prop_assert_eq!(since, Elapsed::from(Duration::from_millis(later.saturating_sub(earlier))));
    }

    #[test]
    fn an_elapsed_time_shows_its_whole_milliseconds(millis in 0..MOST) {
        prop_assert_eq!(Elapsed::from(Duration::from_millis(millis)).to_string(), millis.to_string());
    }

    #[test]
    fn two_elapsed_times_add_up_to_their_sum(first in 0..MOST, second in 0..MOST) {
        let sum = Elapsed::from(Duration::from_millis(first)) + Elapsed::from(Duration::from_millis(second));
        prop_assert_eq!(sum, Elapsed::from(Duration::from_millis(first + second)));
    }
}

#[test]
fn the_monotonic_clock_moves_forward_with_time() {
    let run = port_core::Run::root();
    let clock = run.port(Monotonic::default());
    let first = clock.now();
    std::thread::sleep(PAUSE);
    assert!(clock.now().since(first) >= Elapsed::from(PAUSE));
}
