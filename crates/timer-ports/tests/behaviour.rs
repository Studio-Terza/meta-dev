use std::time::Duration;

use timer_ports::{Elapsed, Moment};

struct Millis;

impl Millis {
    fn moment(millis: u64) -> Moment {
        Moment::from(Duration::from_millis(millis))
    }

    fn elapsed(millis: u64) -> Elapsed {
        Elapsed::from(Duration::from_millis(millis))
    }
}

#[test]
fn a_moment_since_an_earlier_one_is_the_time_between_them_and_never_negative() {
    assert_eq!(Millis::moment(5_000).since(Millis::moment(2_000)), Millis::elapsed(3_000));
    assert_eq!(Millis::moment(2_000).since(Millis::moment(5_000)), Elapsed::from(Duration::ZERO));
    assert_eq!(Millis::moment(7).since(Millis::moment(7)), Elapsed::from(Duration::ZERO));
}

#[test]
fn an_elapsed_time_shows_its_whole_milliseconds() {
    assert_eq!(Millis::elapsed(1_500).to_string(), "1500");
    assert_eq!(Millis::elapsed(0).to_string(), "0");
}

#[test]
fn two_elapsed_times_add_up_to_their_sum() {
    assert_eq!(Millis::elapsed(1_200) + Millis::elapsed(300), Millis::elapsed(1_500));
    assert_eq!(Millis::elapsed(0) + Millis::elapsed(40), Millis::elapsed(40));
}

#[test]
fn the_monotonic_clock_moves_forward_with_time() {
    let run = port_core::Run::root();
    let clock = run.port(timer_ports::Monotonic::default());
    let first = timer_ports::Timer::now(&clock);
    std::thread::sleep(Duration::from_millis(5));
    let later = timer_ports::Timer::now(&clock);
    assert!(later.since(first) >= Elapsed::from(Duration::from_millis(5)));
}
