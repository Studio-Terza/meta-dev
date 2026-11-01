use lint_no_test_only_api::NoTestOnlyApi;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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

const STAGED: &str = "pub struct A;\nimpl A {\n    pub fn staged() -> A {\n        A\n    }\n}\n";
const STAGED_TEST: &str = "fn t() {\n    A::staged();\n}\n";
const STAGED_SHIPPED: &str = "fn run() {\n    A::staged();\n}\n";
const STAGED_MACRO: &str = "fn run() {\n    show!(A::staged());\n}\n";
const STAGED_INLINE_TEST: &str = "pub struct A;\nimpl A {\n    pub fn staged() -> A {\n        A\n    }\n}\n#[cfg(test)]\nmod tests {\n    fn t() {\n        super::A::staged();\n    }\n}\n";
const STOPPED: &str = "pub enum Clock {\n    Live,\n    Stopped,\n}\n";
const STOPPED_TEST: &str = "fn t() {\n    Clock::Stopped;\n}\n";
const STOPPED_SHIPPED: &str = "fn run() -> Clock {\n    Clock::Stopped\n}\n";
const PRIVATE_STAGED: &str = "struct A;\nimpl A {\n    fn staged() -> A {\n        A\n    }\n}\n";
const TRAIT_STAGED: &str = "pub struct A;\nimpl Default for A {\n    fn default() -> A {\n        A\n    }\n}\n";
const DEFAULT_TEST: &str = "fn t() {\n    A::default();\n}\n";
const FREE_FN: &str = "pub fn staged() {}\n";
const GETTER: &str = "pub struct A;\nimpl A {\n    pub fn staged(&self) -> u8 {\n        1\n    }\n}\n";
const OPTION_CONSTRUCTOR: &str =
    "pub struct A;\nimpl A {\n    pub fn staged() -> Option<A> {\n        None\n    }\n}\n";
const FREE_TEST: &str = "fn t() {\n    staged();\n}\n";

const IMPL_USER: &str = "struct B;\nimpl B {\n    fn run() {\n        A::staged();\n    }\n}\n";
const ARG_USER: &str = "fn run(Clock::Stopped: Clock) {}\n";
const DISCRIMINANT_USER: &str = "enum B {\n    X = Clock::Stopped as isize,\n}\n";
const NESTED: &str = "mod inner {\n    pub struct A;\n    impl A {\n        pub fn staged() -> A {\n            A\n        }\n    }\n}\n";
const RECEIVER_SELF: &str = "pub struct A;\nimpl A {\n    pub fn staged(&self) -> Self {\n        A\n    }\n}\n";
const PLAIN_RETURN: &str = "pub struct A;\nimpl A {\n    pub fn staged() -> u8 {\n        1\n    }\n}\n";
const SELF_RETURN: &str = "pub struct A;\nimpl A {\n    pub fn staged() -> Self {\n        A\n    }\n}\n";

const CASES: [Case; 41] = [
    Case::new(
        Title::new("shipped caller inside an impl"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STAGED)),
            Source::new(Located::new("src/b.rs"), Text::new(IMPL_USER)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("variant named in an argument pattern"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STOPPED)),
            Source::new(Located::new("src/b.rs"), Text::new(ARG_USER)),
            Source::new(Located::new("tests/a.rs"), Text::new(STOPPED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("variant named in a discriminant"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STOPPED)),
            Source::new(Located::new("src/b.rs"), Text::new(DISCRIMINANT_USER)),
            Source::new(Located::new("tests/a.rs"), Text::new(STOPPED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor in an inline module"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(NESTED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&["src/a.rs:4"]),
    ),
    Case::new(
        Title::new("method with a receiver returning self"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(RECEIVER_SELF)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("fn with no receiver returning another type"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(PLAIN_RETURN)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor returning self"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(SELF_RETURN)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("constructor only tests call"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STAGED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("constructor with a shipped caller"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STAGED)),
            Source::new(Located::new("src/b.rs"), Text::new(STAGED_SHIPPED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor called inside a macro"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STAGED)),
            Source::new(Located::new("src/b.rs"), Text::new(STAGED_MACRO)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor nobody calls"),
        &[Source::new(Located::new("src/a.rs"), Text::new(STAGED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor called in a test module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(STAGED_INLINE_TEST))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("variant only tests name"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STOPPED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STOPPED_TEST)),
        ],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("variant with a shipped user"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(STOPPED)),
            Source::new(Located::new("src/b.rs"), Text::new(STOPPED_SHIPPED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STOPPED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("private constructor"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(PRIVATE_STAGED)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait method only tests call"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(TRAIT_STAGED)),
            Source::new(Located::new("tests/a.rs"), Text::new(DEFAULT_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("free fn only tests call"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(FREE_FN)),
            Source::new(Located::new("tests/a.rs"), Text::new(FREE_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("method with a receiver only tests call"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(GETTER)),
            Source::new(Located::new("tests/a.rs"), Text::new(FREE_TEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("constructor returning an option"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(OPTION_CONSTRUCTOR)),
            Source::new(Located::new("tests/a.rs"), Text::new(STAGED_TEST)),
        ],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("gated crate fn"),
        &[Source::new(Located::new("src/a.rs"), Text::new(GATED_CRATE_FN))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("gated any ships elsewhere"),
        &[Source::new(Located::new("src/a.rs"), Text::new(GATED_ANY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("gated method"),
        &[Source::new(Located::new("src/a.rs"), Text::new(GATED_METHOD))],
        Flagged::new(&["src/a.rs:4"]),
    ),
    Case::new(
        Title::new("gated impl"),
        &[Source::new(Located::new("src/a.rs"), Text::new(GATED_IMPL))],
        Flagged::new(&["src/a.rs:4"]),
    ),
    Case::new(
        Title::new("gated field"),
        &[Source::new(Located::new("src/a.rs"), Text::new(GATED_FIELD))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(Title::new("not test"), &[Source::new(Located::new("src/a.rs"), Text::new(NOT_TEST))], Flagged::new(&[])),
    Case::new(
        Title::new("hidden method"),
        &[Source::new(Located::new("src/a.rs"), Text::new(HIDDEN_METHOD))],
        Flagged::new(&["src/a.rs:4"]),
    ),
    Case::new(
        Title::new("hidden private"),
        &[Source::new(Located::new("src/a.rs"), Text::new(HIDDEN_PRIVATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("doc string"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DOCUMENTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("gated in tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GATED_FN))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("feature in any"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ANY_FEATURE))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("feature in cfg_attr"),
        &[Source::new(Located::new("src/a.rs"), Text::new(FEATURE_ATTRIBUTE))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("feature in cfg macro"),
        &[Source::new(Located::new("src/a.rs"), Text::new(FEATURE_MACRO))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("feature in tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(FEATURE))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("other feature"),
        &[Source::new(Located::new("src/a.rs"), Text::new(OTHER_FEATURE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("every kind of item"),
        &[Source::new(Located::new("src/a.rs"), Text::new(EVERY_KIND))],
        Flagged::new(&[
            "src/a.rs:2",
            "src/a.rs:4",
            "src/a.rs:6",
            "src/a.rs:8",
            "src/a.rs:10",
            "src/a.rs:12",
            "src/a.rs:14",
            "src/a.rs:16",
            "src/a.rs:18",
        ]),
    ),
    Case::new(
        Title::new("impl members"),
        &[Source::new(Located::new("src/a.rs"), Text::new(MEMBERS))],
        Flagged::new(&["src/a.rs:4", "src/a.rs:6"]),
    ),
    Case::new(Title::new("a doc alias"), &[Source::new(Located::new("src/a.rs"), Text::new(ALIAS))], Flagged::new(&[])),
    Case::new(
        Title::new("hidden in another attribute"),
        &[Source::new(Located::new("src/a.rs"), Text::new(OTHER_LIST))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(INLINE))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("feature in another attribute"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ALLOW_FEATURE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("another cfg key"),
        &[Source::new(Located::new("src/a.rs"), Text::new(OTHER_KEY))],
        Flagged::new(&[]),
    ),
];

#[test]
fn test_only_api_is_flagged() {
    assert_eq!(Sources::mismatches::<NoTestOnlyApi>(&CASES), Lines::default());
}
