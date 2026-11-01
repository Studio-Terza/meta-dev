use lint_no_numbers_in_binaries::NoNumbersInBinaries;

use lint_fixtures::{Case, Source, Sources};

const NUMBERS: &str = "fn main() {\n    let retries = 5;\n    let ratio = 0.5;\n}\n";
const UNITS: &str = "fn main() {\n    let first = 1;\n    let none = 0;\n}\n";
const MODULE: &str = "fn main() {}\nmod wiring;\n";
const WIRING: &str = "fn pause() -> u32 {\n    30\n}\n";

const UNIT_FLOAT: &str = "fn main() {\n    drop(1.0);\n}\n";

const CASES: [Case; 6] = [
    Case::new("binary", &[Source::new("src/main.rs", NUMBERS)], &["src/main.rs:2", "src/main.rs:3"]),
    Case::new("units", &[Source::new("src/main.rs", UNITS)], &[]),
    Case::new("library", &[Source::new("src/lib.rs", NUMBERS)], &[]),
    Case::new(
        "binary module",
        &[Source::new("src/main.rs", MODULE), Source::new("src/wiring.rs", WIRING)],
        &["src/wiring.rs:2"],
    ),
    Case::new("other binary", &[Source::new("src/bin/tool.rs", NUMBERS)], &["src/bin/tool.rs:2", "src/bin/tool.rs:3"]),
    Case::new("unit float", &[Source::new("src/main.rs", UNIT_FLOAT)], &[]),
];

#[test]
fn numbers_in_a_binary_are_flagged() {
    assert_eq!(Sources::mismatches::<NoNumbersInBinaries>(&CASES), Vec::<String>::new());
}
