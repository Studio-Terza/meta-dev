mod common;

use common::Sources;
use lints::NamedLifetimes;

const SHORT: &str = "struct A<'a>(&'a str);\nfn f<'b>(x: &'b str) {}\n";
const NAMED: &str = "struct A<'text>(&'text str);\nfn f(x: &'static str) {}\n";

#[test]
fn single_letter_lifetimes_are_reported() {
    let report = NamedLifetimes::report(&[Sources::file("a.rs", SHORT)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:2"]);
    assert!(report.violations().iter().any(|violation| violation.message() == "'a"));
}

#[test]
fn word_lifetimes_pass() {
    assert!(NamedLifetimes::report(&[Sources::file("a.rs", NAMED)]).is_clean());
}

#[test]
fn a_one_letter_non_ascii_lifetime_is_reported() {
    let report = NamedLifetimes::report(&[Sources::file("a.rs", "struct A<'é>(&'é str);\n")]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}
