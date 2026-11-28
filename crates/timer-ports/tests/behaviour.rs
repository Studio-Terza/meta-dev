use std::time::Duration;

use pipeline::Measure;
use port_core::{Calls, Run};
use timer_ports::{Clock, Deadline, Due, Elapsed, Pause, SystemClock, Timeout, Wall, clock_ops};

const SECOND: i64 = 1_000_000_000;

#[test]
fn walls_measure_never_negative_spans_that_add_and_show_whole_milliseconds() {
    let [early, late] = [2, 5].map(|seconds| Wall::new(seconds * SECOND));
    let spans =
        [late.since(early), early.since(late), late.since(early).joined(Elapsed::from(Duration::from_millis(40)))];
    assert_eq!(spans.map(|span| span.to_string()), ["3000", "0", "3040"]);
    assert_eq!(format!("{:>6}|{}", Elapsed::from(Duration::from_secs(5)), Elapsed::default()), "  5000|0");
    assert_eq!([Wall::from_seconds(2.5_f64), Wall::from_seconds(-1.0_f64)], [Some(Wall::new(5 * SECOND / 2)), None]);
}

#[test]
fn a_deadline_is_pending_before_its_timeout_and_passed_from_it_on() {
    let start = Wall::new(100 * SECOND);
    let deadline = Deadline::after(start, Timeout::new(60));
    let at = [59, 60, 61].map(|seconds| deadline.due(Wall::new((100 + seconds) * SECOND)));
    assert_eq!(at, [Due::Pending, Due::Passed, Due::Passed]);
    assert_eq!(start.after(Elapsed::from(Duration::MAX)), Wall::new(i64::MAX));
}

#[test]
fn the_system_clock_moves_forward_by_at_least_a_pause() {
    let run = Run::root();
    let forward = {
        let clock = run.port(SystemClock);
        let first = clock.now();
        let paused = clock.paused(Pause::new(1));
        [paused.since(first) >= Elapsed::from(Duration::from_secs(1)), clock.now() >= paused]
    };
    assert_eq!(forward, [true, true]);
    let measured = run.finish();
    let snapshot = measured.snapshot();
    assert_eq!([snapshot.calls::<clock_ops::Now>(), snapshot.calls::<clock_ops::Paused>()], [2, 1].map(Calls::new));
}
