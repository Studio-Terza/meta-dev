use std::ffi::OsStr;

use golden::{Difference, Failure, Mode, golden};

struct Shown;

impl Shown {
    const NO_DIFFERENCE: &str = "no difference\n";

    fn between(recorded: &str, actual: &str) -> String {
        Difference::between(recorded, actual)
            .map_or(Self::NO_DIFFERENCE.to_owned(), |difference| difference.to_string())
    }
}

#[test]
fn a_changed_line_shows_its_context() -> Result<(), Failure> {
    let recorded = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\n";
    let actual = "one\ntwo\nthree\nfour\nFIVE\nsix\nseven\neight\nnine\n";
    golden!("changed_line.diff", &Shown::between(recorded, actual))
}

#[test]
fn a_missing_final_newline_is_visible() -> Result<(), Failure> {
    golden!("final_newline.diff", &Shown::between("same\n", "same"))
}

#[test]
fn trailing_space_and_carriage_returns_are_visible() -> Result<(), Failure> {
    golden!("invisible.diff", &Shown::between("a\nb\n", "a \r\nb\n"))
}

#[test]
fn added_lines_appear_only_on_the_actual_side() -> Result<(), Failure> {
    golden!("added.diff", &Shown::between("a\nc\n", "a\nb\nc\n"))
}

#[test]
fn a_repeated_line_is_not_counted_twice() -> Result<(), Failure> {
    golden!("repeated.diff", &Shown::between("a\na\n", "a\n"))
}

#[test]
fn an_empty_golden_shows_every_actual_line() -> Result<(), Failure> {
    golden!("empty.diff", &Shown::between("", "a\nb\n"))
}

#[test]
fn a_refused_recording_says_how_to_record() -> Result<(), Failure> {
    let refused = |record, ci| {
        Mode::chosen(Some(OsStr::new(record)), ci).map_or_else(|failure| failure.to_string(), |_| String::new())
    };
    golden!("refused.txt", &format!("{}\n{}\n", refused("yes", None), refused("1", Some(OsStr::new("true")))))
}
