use lint_core::Lint;
use lint_fixtures::{Case, Source, Sources};
use lint_mandatory_laws::MandatoryLaws;

const TEXT_PAIR: &str = "struct A;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for A {}\n";
const DISPLAY_ONLY: &str = "struct A;\nimpl std::fmt::Display for A {}\n";
const SERDE_PAIR: &str = "#[derive(Serialize, Deserialize)]\nstruct A;\n";
const SERDE_ONE: &str = "#[derive(Serialize)]\nstruct A;\n";
const ENCODING: &str = "struct A;\nimpl A {\n    fn encode(&self) {}\n    fn decode() {}\n}\n";
const FORMS: &str = "struct A;\nimpl A {\n    fn to_hex(&self) {}\n    fn from_hex() {}\n}\n";
const MISMATCHED_FORMS: &str = "struct A;\nimpl A {\n    fn to_hex(&self) {}\n    fn from_bits() {}\n}\n";
const ORDER: &str = "struct A;\nimpl Ord for A {}\n";
const EQUALITY: &str = "struct A;\nimpl PartialEq for A {}\n";
const DERIVED_ORDER: &str = "#[derive(PartialEq, Ord)]\nstruct A;\n";
const ROUNDTRIP: &str = "laws::roundtrip!(a, any::<A>(), A::encode, A::decode);\n";
const LAW: &str = "laws::law!(a, any::<A>(), value, { check(value) });\n";
const PROPTEST: &str = "proptest! {\n    #[test]\n    fn a(value in any::<A>()) {}\n}\n";
const OTHER_TYPE: &str = "laws::roundtrip!(b, any::<B>(), B::encode, B::decode);\n";

const SERDE_ENUM: &str = "#[derive(Serialize, Deserialize)]\nenum A {}\n";
const SPLIT_TYPES: &str = "struct A;\nstruct B;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for B {}\n";
const UNRELATED_MACRO: &str = "fn t() {\n    vec![A];\n}\n";

const CASES: [Case; 16] = [
    Case::new("display and from_str", &[Source::new("src/a.rs", TEXT_PAIR)], &["src/a.rs:1"]),
    Case::new("display alone", &[Source::new("src/a.rs", DISPLAY_ONLY)], &[]),
    Case::new("serde pair", &[Source::new("src/a.rs", SERDE_PAIR)], &["src/a.rs:2"]),
    Case::new("serde enum", &[Source::new("src/a.rs", SERDE_ENUM)], &["src/a.rs:2"]),
    Case::new("pair split across types", &[Source::new("src/a.rs", SPLIT_TYPES)], &[]),
    Case::new(
        "unrelated macro",
        &[Source::new("src/a.rs", ORDER), Source::new("tests/a.rs", UNRELATED_MACRO)],
        &["src/a.rs:1"],
    ),
    Case::new("serialize alone", &[Source::new("src/a.rs", SERDE_ONE)], &[]),
    Case::new("encode and decode", &[Source::new("src/a.rs", ENCODING)], &["src/a.rs:1"]),
    Case::new("to and from", &[Source::new("src/a.rs", FORMS)], &["src/a.rs:1"]),
    Case::new("to and from of different forms", &[Source::new("src/a.rs", MISMATCHED_FORMS)], &[]),
    Case::new("hand-written equality", &[Source::new("src/a.rs", EQUALITY)], &["src/a.rs:1"]),
    Case::new("derived order", &[Source::new("src/a.rs", DERIVED_ORDER)], &[]),
    Case::new("roundtrip in a test", &[Source::new("src/a.rs", ENCODING), Source::new("tests/a.rs", ROUNDTRIP)], &[]),
    Case::new("law covers order", &[Source::new("src/a.rs", ORDER), Source::new("tests/a.rs", LAW)], &[]),
    Case::new(
        "proptest covers equality",
        &[Source::new("src/a.rs", EQUALITY), Source::new("tests/a.rs", PROPTEST)],
        &[],
    ),
    Case::new(
        "law for another type",
        &[Source::new("src/a.rs", ENCODING), Source::new("tests/a.rs", OTHER_TYPE)],
        &["src/a.rs:1"],
    ),
];

const LAW_FOR_PAIR: &str = "laws::law!(a, any::<A>(), value, { check(value) });\n";

#[test]
fn missing_laws_are_flagged() {
    assert_eq!(Sources::mismatches::<MandatoryLaws>(&CASES), Vec::<String>::new());
}

#[test]
fn a_plain_law_does_not_stand_in_for_a_roundtrip() {
    let report =
        MandatoryLaws::report(&[Sources::file("src/a.rs", ENCODING), Sources::file("tests/a.rs", LAW_FOR_PAIR)]);
    assert!(!report.is_clean());
}

#[test]
fn the_message_names_the_fix() {
    let pair = MandatoryLaws::report(&[Sources::file("src/a.rs", ENCODING)]);
    let order = MandatoryLaws::report(&[Sources::file("src/a.rs", ORDER)]);
    assert!(pair.violations().iter().all(|violation| violation.message().contains("laws::roundtrip!")));
    assert!(order.violations().iter().all(|violation| violation.message().contains("laws::law!")));
    assert!(!pair.is_clean() && !order.is_clean());
}
