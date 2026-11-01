use lint_core::Lint;
use lint_fixtures::{Case, Source, Sources};
use lint_thin_main::ThinMain;

const FAT: &str = "use std::fmt;\nfn main() {}\nstruct Extra;\n";
const THIN: &str = "use std::fmt;\nfn main() {}\n";

const BINARY_ROOTS: [Case; 2] = [
    Case::new("main", &[Source::new("crates/x/src/main.rs", FAT)], &["crates/x/src/main.rs:3"]),
    Case::new("bin directory", &[Source::new("src/bin/tool.rs", FAT)], &["src/bin/tool.rs:3"]),
];

#[test]
fn logic_in_a_binary_root_is_reported() {
    assert_eq!(Sources::mismatches::<ThinMain>(&BINARY_ROOTS), Vec::<String>::new());
}

#[test]
fn a_thin_main_passes_clean() {
    assert!(ThinMain::report(&[Sources::file("src/main.rs", THIN)]).is_clean());
}

#[test]
fn files_other_than_the_binary_root_are_not_checked() {
    assert!(ThinMain::report(&[Sources::file("src/lib.rs", FAT)]).is_clean());
}

#[test]
fn an_item_is_reported_on_its_own_line_not_its_attribute_line() {
    let attributed = "#[derive(Debug)]\n#[allow(dead_code)]\nstruct Extra;\nfn main() {}\n";
    let report = ThinMain::report(&[Sources::file("src/main.rs", attributed)]);
    assert_eq!(Sources::lines(&report), ["src/main.rs:3"]);
}
