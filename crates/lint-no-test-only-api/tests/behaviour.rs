use lint_no_test_only_api::NoTestOnlyApi;

use lint_fixtures::{Case, Source, Sources};

const GATED_FN: &str = "#[cfg(test)]\npub fn f() {}\n";
const GATED_CRATE_FN: &str = "#[cfg(test)]\npub(crate) fn f() {}\n";
const GATED_ANY: &str = "#[cfg(any(test, unix))]\npub struct A;\n";
const GATED_METHOD: &str = "struct A;\nimpl A {\n    #[cfg(test)]\n    pub fn f() {}\n}\n";
const GATED_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    pub fn f() {}\n}\n";
const GATED_FIELD: &str = "struct A {\n    #[cfg(test)]\n    pub a: u8,\n}\n";
const NOT_TEST: &str = "#[cfg(not(test))]\npub fn f() {}\n";
const HIDDEN_METHOD: &str = "struct A;\nimpl A {\n    #[doc(hidden)]\n    pub fn f() {}\n}\n";
const HIDDEN_PRIVATE: &str = "#[doc(hidden)]\nfn f() {}\n";
const DOCUMENTED: &str = "#[doc = \"hidden\"]\npub fn f() {}\n";
const FEATURE: &str = "#[cfg(feature = \"test-utils\")]\nfn f() {}\n";
const ANY_FEATURE: &str = "#[cfg(any(unix, feature = \"mocks\"))]\nfn f() {}\n";
const FEATURE_ATTRIBUTE: &str = "#[cfg_attr(feature = \"testing\", derive(Debug))]\nstruct A;\n";
const FEATURE_MACRO: &str = "fn f() -> bool {\n    cfg!(feature = \"test_helpers\")\n}\n";
const OTHER_FEATURE: &str = "#[cfg(feature = \"latest\")]\nfn f() {}\n#[cfg(feature = \"contest\")]\nfn g() {}\n";

const EVERY_KIND: &str = "#[cfg(test)]\npub const A: u8 = 1;\n#[cfg(test)]\npub enum B {}\n#[cfg(test)]\npub mod c {}\n#[cfg(test)]\npub static D: u8 = 1;\n#[cfg(test)]\npub trait E {}\n#[cfg(test)]\npub type F = u8;\n#[cfg(test)]\npub union G { x: u8 }\n#[cfg(test)]\npub use std::fmt;\n#[cfg(test)]\npub struct H;\n";
const MEMBERS: &str =
    "struct A;\nimpl A {\n    #[cfg(test)]\n    pub const B: u8 = 1;\n    #[cfg(test)]\n    pub type C = u8;\n}\n";
const ALIAS: &str = "#[doc(alias = \"x\")]\npub fn f() {}\n";
const OTHER_LIST: &str = "#[allow(hidden)]\npub fn f() {}\n";
const INLINE: &str = "mod inner {\n    #[cfg(test)]\n    pub fn f() {}\n}\n";
const ALLOW_FEATURE: &str = "#[allow(feature = \"test-utils\")]\nfn f() {}\n";
const OTHER_KEY: &str = "#[cfg(target_os = \"test-utils\")]\nfn f() {}\n";

const CASES: [Case; 22] = [
    Case::new("gated crate fn", &[Source::new("src/a.rs", GATED_CRATE_FN)], &["src/a.rs:2"]),
    Case::new("gated any ships elsewhere", &[Source::new("src/a.rs", GATED_ANY)], &[]),
    Case::new("gated method", &[Source::new("src/a.rs", GATED_METHOD)], &["src/a.rs:4"]),
    Case::new("gated impl", &[Source::new("src/a.rs", GATED_IMPL)], &["src/a.rs:4"]),
    Case::new("gated field", &[Source::new("src/a.rs", GATED_FIELD)], &["src/a.rs:3"]),
    Case::new("not test", &[Source::new("src/a.rs", NOT_TEST)], &[]),
    Case::new("hidden method", &[Source::new("src/a.rs", HIDDEN_METHOD)], &["src/a.rs:4"]),
    Case::new("hidden private", &[Source::new("src/a.rs", HIDDEN_PRIVATE)], &[]),
    Case::new("doc string", &[Source::new("src/a.rs", DOCUMENTED)], &[]),
    Case::new("gated in tests", &[Source::new("tests/a.rs", GATED_FN)], &[]),
    Case::new("feature in any", &[Source::new("src/a.rs", ANY_FEATURE)], &["src/a.rs:1"]),
    Case::new("feature in cfg_attr", &[Source::new("src/a.rs", FEATURE_ATTRIBUTE)], &["src/a.rs:1"]),
    Case::new("feature in cfg macro", &[Source::new("src/a.rs", FEATURE_MACRO)], &["src/a.rs:2"]),
    Case::new("feature in tests", &[Source::new("tests/a.rs", FEATURE)], &["tests/a.rs:1"]),
    Case::new("other feature", &[Source::new("src/a.rs", OTHER_FEATURE)], &[]),
    Case::new(
        "every kind of item",
        &[Source::new("src/a.rs", EVERY_KIND)],
        &[
            "src/a.rs:2",
            "src/a.rs:4",
            "src/a.rs:6",
            "src/a.rs:8",
            "src/a.rs:10",
            "src/a.rs:12",
            "src/a.rs:14",
            "src/a.rs:16",
            "src/a.rs:18",
        ],
    ),
    Case::new("impl members", &[Source::new("src/a.rs", MEMBERS)], &["src/a.rs:4", "src/a.rs:6"]),
    Case::new("a doc alias", &[Source::new("src/a.rs", ALIAS)], &[]),
    Case::new("hidden in another attribute", &[Source::new("src/a.rs", OTHER_LIST)], &[]),
    Case::new("inline module", &[Source::new("src/a.rs", INLINE)], &["src/a.rs:3"]),
    Case::new("feature in another attribute", &[Source::new("src/a.rs", ALLOW_FEATURE)], &[]),
    Case::new("another cfg key", &[Source::new("src/a.rs", OTHER_KEY)], &[]),
];

#[test]
fn test_only_api_is_flagged() {
    assert_eq!(Sources::mismatches::<NoTestOnlyApi>(&CASES), Vec::<String>::new());
}
