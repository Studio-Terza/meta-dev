use lints::NoNumbersInBinaries;

use crate::common::{Case, Source, Sources};

const NUMBERS: &str = "fn main() {\n    let retries = 5;\n    let ratio = 0.5;\n}\n";
const UNITS: &str = "fn main() {\n    let first = 1;\n    let none = 0;\n}\n";
const MODULE: &str = "fn main() {}\nmod wiring;\n";
const WIRING: &str = "fn pause() -> u32 {\n    30\n}\n";

const CASES: [Case; 5] = [
    Case {
        name: "binary",
        sources: &[Source::new("src/main.rs", NUMBERS)],
        flagged: &["src/main.rs:2", "src/main.rs:3"],
    },
    Case { name: "units", sources: &[Source::new("src/main.rs", UNITS)], flagged: &[] },
    Case { name: "library", sources: &[Source::new("src/lib.rs", NUMBERS)], flagged: &[] },
    Case {
        name: "binary module",
        sources: &[Source::new("src/main.rs", MODULE), Source::new("src/wiring.rs", WIRING)],
        flagged: &["src/wiring.rs:2"],
    },
    Case { name: "other binary", sources: &[Source::new("src/bin/tool.rs", NUMBERS)], flagged: &[] },
];

#[test]
fn numbers_in_a_binary_are_flagged() {
    assert_eq!(Sources::mismatches::<NoNumbersInBinaries>(&CASES), Vec::<String>::new());
}
