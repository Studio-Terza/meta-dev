use lint_core::Lint;
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use lint_mandatory_laws::MandatoryLaws;

const TEXT_PAIR: &str = "pub struct A;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for A {}\n";
const DISPLAY_ONLY: &str = "pub struct A;\nimpl std::fmt::Display for A {}\n";
const SERDE_PAIR: &str = "#[derive(Serialize, Deserialize)]\npub struct A;\n";
const SERDE_ONE: &str = "#[derive(Serialize)]\npub struct A;\n";
const ENCODING: &str = "pub struct A;\nimpl A {\n    fn encode(&self) {}\n    fn decode() {}\n}\n";
const FORMS: &str = "pub struct A;\nimpl A {\n    fn to_hex(&self) {}\n    fn from_hex() {}\n}\n";
const MISMATCHED_FORMS: &str = "pub struct A;\nimpl A {\n    fn to_hex(&self) {}\n    fn from_bits() {}\n}\n";
const ORDER: &str = "pub struct A;\nimpl Ord for A {}\n";
const EQUALITY: &str = "pub struct A;\nimpl PartialEq for A {}\n";
const DERIVED_ORDER: &str = "#[derive(PartialEq, Ord)]\npub struct A;\n";
const POINTER_EQUALITY: &str = "pub struct A;\nimpl PartialEq for A {\n    fn eq(&self, other: &Self) -> bool {\n        std::rc::Rc::ptr_eq(&self.0, &other.0)\n    }\n}\n";
const POINTER_FUNCTION: &str = "pub struct A;\nimpl PartialEq for A {\n    fn eq(&self, other: &Self) -> bool {\n        std::ptr::eq(self, other)\n    }\n}\n";
const POINTER_AND_FIELD: &str = "pub struct A;\nimpl PartialEq for A {\n    fn eq(&self, other: &Self) -> bool {\n        Rc::ptr_eq(&self.0, &other.0) && self.1 == other.1\n    }\n}\n";
const POINTER_ORDER: &str = "pub struct A;\nimpl Ord for A {\n    fn cmp(&self, other: &Self) -> Ordering {\n        Rc::ptr_eq(&self.0, &other.0).cmp(&true)\n    }\n}\n";
const ROUNDTRIP: &str = "laws::roundtrip!(a, any::<A>(), A::encode, A::decode);\n";
const LAW: &str = "laws::law!(a, any::<A>(), value, { check(value) });\n";
const PROPTEST: &str = "proptest! {\n    #[test]\n    fn a(value in any::<A>()) {}\n}\n";
const OTHER_TYPE: &str = "laws::roundtrip!(b, any::<B>(), B::encode, B::decode);\n";

const SERDE_ENUM: &str = "#[derive(Serialize, Deserialize)]\npub enum A {}\n";
const SPLIT_TYPES: &str =
    "pub struct A;\nstruct B;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for B {}\n";
const UNRELATED_MACRO: &str = "fn t() {\n    vec![A];\n}\n";

const TITLE_ONLY: &str = "laws::roundtrip!(A, any::<u8>(), B::encode, B::decode);\n";
const BOTH_PAIRS: &str = "pub struct A;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for A {}\nimpl A {\n    fn encode(&self) {}\n    fn decode() {}\n}\n";
const CODED_ROUNDTRIP: &str = "laws::roundtrip!(coded, any::<A>(), A::encode, A::decode);\n";
const TEXT_ROUNDTRIP: &str = "laws::roundtrip!(text, any::<A>(), ToString::to_string, A::from_str);\n";
const DISPLAY_FILE: &str = "pub struct A;\nimpl std::fmt::Display for crate::A {}\n";
const FROM_STR_FILE: &str = "impl std::str::FromStr for crate::A {}\n";
const CONDITIONAL_SERDE: &str = "#[cfg_attr(feature = \"serde\", derive(Serialize, Deserialize))]\npub struct A;\n";
const CODEC: &str = "pub struct A;\ntrait Codec {\n    fn encode(&self);\n    fn decode();\n}\nimpl Codec for A {\n    fn encode(&self) {}\n    fn decode() {}\n}\n";
const KEY: &str = "pub struct Key;\nimpl Key {\n    pub fn encode(&self) {}\n    pub fn decode() {}\n}\n";
const KEY_ROUNDTRIP: &str = "laws::roundtrip!(key_recovers, any::<Key>(), Key::encode, Key::decode);\n";

const ATTRIBUTED_TITLE: &str = "laws::roundtrip!(#[allow(unused)] A, any::<u8>(), B::encode, B::decode);\n";
const HALF_ROUNDTRIP: &str = "laws::roundtrip!(half, any::<A>(), A::encode, other);\n";
const FOREIGN_ROUNDTRIP: &str = "other::roundtrip!(foreign, any::<A>(), A::encode, A::decode);\n";
const LAW_NAMING_BOTH: &str = "laws::law!(both, any::<A>(), value, { A::decode(&value.encode()) });\n";
const ENCODE_ONLY: &str = "pub struct A;\nimpl A {\n    fn encode(&self) {}\n}\n";
const DECODE_ONLY: &str = "pub struct A;\nimpl A {\n    fn decode() {}\n}\n";
const PRIVATE_TEXT_PAIR: &str = "struct A;\nimpl std::fmt::Display for A {}\nimpl std::str::FromStr for A {}\n";
const CRATE_ORDER: &str = "pub(crate) struct A;\nimpl Ord for A {}\n";
const PRIVATE_TYPE_PUBLIC_IMPL: &str = "struct A;\nimpl A {\n    pub fn encode(&self) {}\n    pub fn decode() {}\n}\n";
const OTHER_ATTRIBUTE_LIST: &str = "#[cfg_attr(feature = \"x\", other(Serialize, Deserialize))]\npub struct A;\n";

const CASES: [Case; 41] = [
    Case::new(
        Title::new("equality by pointer"),
        &[Source::new(Located::new("src/a.rs"), Text::new(POINTER_EQUALITY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("equality by ptr eq"),
        &[Source::new(Located::new("src/a.rs"), Text::new(POINTER_FUNCTION))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("pointer and a field"),
        &[Source::new(Located::new("src/a.rs"), Text::new(POINTER_AND_FIELD))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("order from a pointer test"),
        &[Source::new(Located::new("src/a.rs"), Text::new(POINTER_ORDER))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("display and from_str"),
        &[Source::new(Located::new("src/a.rs"), Text::new(TEXT_PAIR))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("display alone"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DISPLAY_ONLY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("serde pair"),
        &[Source::new(Located::new("src/a.rs"), Text::new(SERDE_PAIR))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("serde enum"),
        &[Source::new(Located::new("src/a.rs"), Text::new(SERDE_ENUM))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("pair split across types"),
        &[Source::new(Located::new("src/a.rs"), Text::new(SPLIT_TYPES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("unrelated macro"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ORDER)),
            Source::new(Located::new("tests/a.rs"), Text::new(UNRELATED_MACRO)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("serialize alone"),
        &[Source::new(Located::new("src/a.rs"), Text::new(SERDE_ONE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("encode and decode"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ENCODING))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("to and from"),
        &[Source::new(Located::new("src/a.rs"), Text::new(FORMS))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("to and from of different forms"),
        &[Source::new(Located::new("src/a.rs"), Text::new(MISMATCHED_FORMS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("hand-written equality"),
        &[Source::new(Located::new("src/a.rs"), Text::new(EQUALITY))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("derived order"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DERIVED_ORDER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("roundtrip in a test"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(ROUNDTRIP)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("law covers order"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ORDER)),
            Source::new(Located::new("tests/a.rs"), Text::new(LAW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("proptest covers equality"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(EQUALITY)),
            Source::new(Located::new("tests/a.rs"), Text::new(PROPTEST)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("law for another type"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(OTHER_TYPE)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("the title is not a mention"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(TITLE_ONLY)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("one roundtrip does not cover two pairs"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(BOTH_PAIRS)),
            Source::new(Located::new("tests/a.rs"), Text::new(CODED_ROUNDTRIP)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a roundtrip for each pair"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(BOTH_PAIRS)),
            Source::new(Located::new("tests/a.rs"), Text::new(CODED_ROUNDTRIP)),
            Source::new(Located::new("tests/b.rs"), Text::new(TEXT_ROUNDTRIP)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the text roundtrip alone leaves the methods bare"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(BOTH_PAIRS)),
            Source::new(Located::new("tests/a.rs"), Text::new(TEXT_ROUNDTRIP)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a law in another package"),
        &[
            Source::new(Located::new("crates/one/src/lib.rs"), Text::new(KEY)),
            Source::new(Located::new("crates/two/src/lib.rs"), Text::new(KEY)),
            Source::new(Located::new("crates/two/tests/t.rs"), Text::new(KEY_ROUNDTRIP)),
        ],
        Flagged::new(&["crates/one/src/lib.rs:1"]),
    ),
    Case::new(
        Title::new("a law in each package"),
        &[
            Source::new(Located::new("crates/one/src/lib.rs"), Text::new(KEY)),
            Source::new(Located::new("crates/one/tests/t.rs"), Text::new(KEY_ROUNDTRIP)),
            Source::new(Located::new("crates/two/src/lib.rs"), Text::new(KEY)),
            Source::new(Located::new("crates/two/tests/t.rs"), Text::new(KEY_ROUNDTRIP)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a pair split across files"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(DISPLAY_FILE)),
            Source::new(Located::new("src/b.rs"), Text::new(FROM_STR_FILE)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("serde behind cfg_attr"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CONDITIONAL_SERDE))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("a codec trait"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CODEC))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a codec trait with a roundtrip"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(CODEC)),
            Source::new(Located::new("tests/a.rs"), Text::new(CODED_ROUNDTRIP)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a roundtrip naming one method"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(HALF_ROUNDTRIP)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a roundtrip from another crate"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(FOREIGN_ROUNDTRIP)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a plain law naming both methods"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(LAW_NAMING_BOTH)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("a title with an attribute"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(ATTRIBUTED_TITLE)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("encode alone"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ENCODE_ONLY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("decode alone"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DECODE_ONLY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("derive-like attribute under cfg_attr"),
        &[Source::new(Located::new("src/a.rs"), Text::new(OTHER_ATTRIBUTE_LIST))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a private type with a conversion pair"),
        &[Source::new(Located::new("src/a.rs"), Text::new(PRIVATE_TEXT_PAIR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a crate-private type with a hand-written order"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CRATE_ORDER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a private type with public methods"),
        &[Source::new(Located::new("src/a.rs"), Text::new(PRIVATE_TYPE_PUBLIC_IMPL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a roundtrip for the right type only"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(ENCODING)),
            Source::new(Located::new("tests/a.rs"), Text::new(OTHER_TYPE)),
        ],
        Flagged::new(&["src/a.rs:1"]),
    ),
];

const LAW_FOR_PAIR: &str = "laws::law!(a, any::<A>(), value, { check(value) });\n";

#[test]
fn missing_laws_are_flagged() {
    assert_eq!(Sources::mismatches::<MandatoryLaws>(&CASES), Lines::default());
}

#[test]
fn a_plain_law_does_not_stand_in_for_a_roundtrip() {
    let report = MandatoryLaws::report(&[
        Sources::file("src/a.rs", Text::new(ENCODING)),
        Sources::file("tests/a.rs", Text::new(LAW_FOR_PAIR)),
    ]);
    assert!(!report.is_clean());
}

#[test]
fn the_message_names_the_fix() {
    let pair = MandatoryLaws::report(&[Sources::file("src/a.rs", Text::new(ENCODING))]);
    let order = MandatoryLaws::report(&[Sources::file("src/a.rs", Text::new(ORDER))]);
    assert!(pair.violations().iter().all(|violation| violation.to_string().contains("laws::roundtrip!")));
    assert!(pair.violations().iter().all(|violation| violation.to_string().contains("`encode` and `decode`")));
    assert!(order.violations().iter().all(|violation| violation.to_string().contains("laws::law!")));
    assert!(!pair.is_clean() && !order.is_clean());
}
