use std::ffi::OsStr;

use golden::{Difference, Failure, Golden, Mode, golden};

struct Row {
    golden: &'static str,
    recorded: &'static str,
    actual: &'static str,
}

impl Row {
    const fn new(golden: &'static str, recorded: &'static str, actual: &'static str) -> Row {
        Row { golden, recorded, actual }
    }
}

struct Shown;

impl Shown {
    const NO_DIFFERENCE: &str = "no difference\n";

    fn between(recorded: &str, actual: &str) -> String {
        Difference::between(recorded, actual)
            .map_or(Self::NO_DIFFERENCE.to_owned(), |difference| difference.to_string())
    }

    fn golden_mismatch(row: &Row) -> String {
        Golden::in_package(env!("CARGO_MANIFEST_DIR"), row.golden)
            .and_then(|golden| golden.verify(&Self::between(row.recorded, row.actual)))
            .map_or_else(|failure| failure.to_string(), |()| String::new())
    }
}

#[test]
fn a_changed_line_shows_its_context() -> Result<(), Failure> {
    let recorded = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\n";
    let actual = "one\ntwo\nthree\nfour\nFIVE\nsix\nseven\neight\nnine\n";
    golden!("changed_line.diff", &Shown::between(recorded, actual))
}

laws::table!(
    each_difference_matches_its_golden,
    Shown::golden_mismatch,
    [
        Row::new("final_newline.diff", "same\n", "same") => "",
        Row::new("invisible.diff", "a\nb\n", "a \r\nb\n") => "",
        Row::new("added.diff", "a\nc\n", "a\nb\nc\n") => "",
        Row::new("repeated.diff", "a\na\n", "a\n") => "",
        Row::new("empty.diff", "", "a\nb\n") => "",
    ]
);

#[test]
fn a_refused_recording_says_how_to_record() -> Result<(), Failure> {
    let refused = |record, ci| {
        Mode::chosen(Some(OsStr::new(record)), ci).map_or_else(|failure| failure.to_string(), |_| String::new())
    };
    golden!("refused.txt", &format!("{}\n{}\n", refused("yes", None), refused("1", Some(OsStr::new("true")))))
}
