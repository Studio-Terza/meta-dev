mod common;

use common::Sources;
use lints::ThinMain;

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
