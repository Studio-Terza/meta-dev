#![deny(clippy::wildcard_enum_match_arm)]

use grammar_test_summary::{Opened, Summary};

struct Summaries;

impl Summaries {
    fn seconds(line: &str) -> Option<f64> {
        Summary::opened(line).and_then(Opened::summary).map(Summary::seconds)
    }

    fn opened(line: &str) -> bool {
        Summary::opened(line).is_some()
    }
}

#[test]
fn the_seconds_are_the_number_after_finished_in() {
    [
        ("test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s", 0.5_f64),
        ("test result: FAILED. 1 passed; 1 failed; finished in 2s", 2.0_f64),
        ("test result: ok. finished in 0.50s", 0.5_f64),
        ("test result: ok. 2 passed; finished in 3s", 3.0_f64),
        ("test result: ok. finished\tin  0.004s", 0.004_f64),
        ("test result: ok. 0 filtered out; finished in 12.25s", 12.25_f64),
    ]
    .into_iter()
    .for_each(|(line, want)| assert_eq!(Summaries::seconds(line), Some(want), "{line}"));
}

#[test]
fn a_line_libtest_could_not_have_written_has_no_seconds() {
    [
        "test result: ok. 1 passed; 0 failed",
        "test result: ok. finished in abc",
        "test result: ok. finished in 0.5",
        "test result: ok. finished in 1s in 2s",
        "test result: ok. finishedin 5s",
        "test result: ok. in 1.5.5s",
        "test result: maybe. finished in 1s",
        "test result: ok. 3 skipped; finished in 1s",
        "test result: ok. finished in 1s ",
        "test result: ok.finished in 1s",
        "running 3 tests",
        "",
    ]
    .into_iter()
    .for_each(|line| assert_eq!(Summaries::seconds(line), None, "{line}"));
}

#[test]
fn a_line_opens_like_a_result_only_when_it_starts_with_the_result_head() {
    [
        ("test result: ok. finished in 1s", true),
        ("test result: garbled", true),
        ("test results: ok. finished in 1s", false),
        (" test result: ok. finished in 1s", false),
        ("test resul: ok", false),
        ("test foo ... ok", false),
        ("", false),
    ]
    .into_iter()
    .for_each(|(line, want)| assert_eq!(Summaries::opened(line), want, "{line}"));
}
