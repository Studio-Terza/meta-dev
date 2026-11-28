use std::thread;
use std::time::Duration;

use fixture::{Failed, Fixture, Properties, Rows};
use port_core::Run;
use proptest::prelude::{Strategy, any, prop_assert_eq, proptest};
use serde::Deserialize;
use timer_ports::{Elapsed, Moment, Monotonic, Stopped, Timer};

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn moments_measure_saturating_spans_that_add_and_show_whole_milliseconds(
        early in any::<u32>().prop_map(|millis| Duration::from_millis(u64::from(millis))),
        late in any::<u32>().prop_map(|millis| Duration::from_millis(u64::from(millis))),
    ) {
        let span = Moment::from(late).since(Moment::from(early));
        prop_assert_eq!(span, Elapsed::from(late.saturating_sub(early)));
        prop_assert_eq!((span + Elapsed::from(early)).to_string(), late.max(early).as_millis().to_string());
        prop_assert_eq!(Moment::from(late).to_string(), late.as_millis().to_string());
    }
}

#[derive(Fixture, Deserialize, Debug)]
struct Paused {
    pause: Duration,
}

#[test]
fn a_monotonic_clock_moves_on_across_a_pause_and_a_stopped_clock_stands_still() -> Result<(), Failed> {
    let run = Run::root();
    let monotonic = run.port(Monotonic::default());
    let stopped = run.port(Stopped);
    Rows::<Paused>::read()?.iter().for_each(|paused| {
        let first = monotonic.now();
        thread::sleep(paused.pause);
        assert!(monotonic.now() > first);
        assert_eq!(stopped.now(), Moment::default());
    });
    Ok(())
}
