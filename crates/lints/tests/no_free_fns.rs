mod common;

use common::Sources;
use lints::NoFreeFns;

const FREE: &str = "fn helper() {}\nmod inner {\n    pub fn nested() {}\n}\n";
const ALLOWED: &str = "fn main() {}\n#[test]\nfn check() {}\nstruct A;\nimpl A {\n    fn method() {}\n}\n";

#[test]
fn free_functions_are_reported_including_nested_modules() {
    let report = NoFreeFns::report(&[Sources::file("a.rs", FREE)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:3"]);
    assert!(report.violations().iter().any(|violation| violation.message() == "fn nested"));
}

#[test]
fn main_tests_and_methods_are_allowed() {
    assert!(NoFreeFns::report(&[Sources::file("a.rs", ALLOWED)]).is_clean());
}
