use crate::common::Sources;
use lints::{Lint, ThinMain};

const FAT: &str = "use std::fmt;\nfn main() {}\nstruct Extra;\n";
const THIN: &str = "use std::fmt;\nfn main() {}\n";

#[test]
fn logic_in_a_binary_root_is_reported() {
    let report = ThinMain::report(&[Sources::file("crates/x/src/main.rs", FAT)]);
    assert_eq!(Sources::lines(&report), ["crates/x/src/main.rs:3"]);
}

#[test]
fn a_thin_main_passes() {
    assert!(ThinMain::report(&[Sources::file("src/main.rs", THIN)]).is_clean());
}

#[test]
fn files_other_than_the_binary_root_are_not_checked() {
    assert!(ThinMain::report(&[Sources::file("src/lib.rs", FAT)]).is_clean());
}

#[test]
fn bin_directory_roots_are_checked() {
    let report = ThinMain::report(&[Sources::file("src/bin/tool.rs", FAT)]);
    assert_eq!(Sources::lines(&report), ["src/bin/tool.rs:3"]);
}

#[test]
fn an_item_is_reported_on_its_own_line_not_its_attribute_line() {
    let attributed = "#[derive(Debug)]\n#[allow(dead_code)]\nstruct Extra;\nfn main() {}\n";
    let report = ThinMain::report(&[Sources::file("src/main.rs", attributed)]);
    assert_eq!(Sources::lines(&report), ["src/main.rs:3"]);
}
