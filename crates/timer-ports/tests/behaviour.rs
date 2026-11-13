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

laws::table!(
    an_elapsed_time_shows_its_whole_milliseconds,
    |milliseconds: &u64| Elapsed::from(Duration::from_millis(*milliseconds)).to_string(),
    [
        1_500 => "1500".to_owned(),
        0 => "0".to_owned(),
    ]
);

laws::table!(
    two_elapsed_times_add_up_to_their_sum,
    |(first, second): &(u64, u64)| Elapsed::from(Duration::from_millis(*first)) + Elapsed::from(Duration::from_millis(*second)),
    [
        (1_200, 300) => Elapsed::from(Duration::from_millis(1_500)),
        (0, 40) => Elapsed::from(Duration::from_millis(40)),
    ]
);

#[test]
fn the_monotonic_clock_moves_forward_with_time() {
    let run = port_core::Run::root();
    let clock = run.port(timer_ports::Monotonic::default());
    let first = timer_ports::Timer::now(&clock);
    std::thread::sleep(Duration::from_millis(5));
    let later = timer_ports::Timer::now(&clock);
    assert!(later.since(first) >= Elapsed::from(Duration::from_millis(5)));
}
