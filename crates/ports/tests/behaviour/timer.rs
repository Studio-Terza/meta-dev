use std::time::Duration;

use ports::timer::{Elapsed, Moment, Monotonic, Timer};

#[test]
fn moments_measure_never_negative_spans_that_add_and_show_whole_milliseconds() {
    let [early, late] = [2_000, 5_000].map(|millis| Moment::from(Duration::from_millis(millis)));
    let spans = [late.since(early), early.since(late), late.since(early) + Elapsed::from(Duration::from_millis(40))];
    assert_eq!(spans.map(|span| span.to_string()), ["3000", "0", "3040"]);
    assert_eq!(format!("{late:>6}|{}", Elapsed::default()), "  5000|0");
    let run = ports::port::Run::root();
    let clock = run.port(Monotonic::default());
    let first = clock.now();
    std::thread::sleep(Duration::from_millis(5));
    assert!(clock.now().since(first) >= Elapsed::from(Duration::from_millis(5)));
}
