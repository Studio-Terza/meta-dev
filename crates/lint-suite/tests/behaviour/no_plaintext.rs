use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_no_plaintext::NoPlaintext;

const TEXT_FIELDS: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A<'a> {\n    owned: String,\n    borrowed: &'a str,\n    cow: Cow<'a, str>,\n    boxed: Box<str>,\n    optional: Option<String>,\n    many: Vec<String>,\n    keyed: BTreeMap<String, u8>,\n    pathed: std::string::String,\n    count: u32,\n    path: PathBuf,\n    other: other::Strand,\n    elsewhere: elsewhere::Id,\n}\n";
const TUPLE_AND_ENUM: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A(String);\n#[derive(Deserialize)]\nenum B {\n    Named(String),\n    Counted(u32),\n    Noted { note: String },\n}\n";
const DERIVE_FORMS: &str = "#[derive(serde::Deserialize)]\nstruct A {\n    a: String,\n}\n#[cfg_attr(feature = \"serde\", derive(serde::Deserialize))]\nstruct B {\n    b: String,\n}\n#[derive(Debug, serde::Deserialize, Clone)]\nstruct C {\n    c: String,\n}\n";
const NOT_DESERIALIZED: &str = "use serde::Serialize;\n#[derive(Debug)]\nstruct A {\n    a: String,\n}\n#[derive(Serialize)]\nstruct B {\n    b: String,\n}\n#[cfg_attr(feature = \"serde\", derive(Serialize))]\nstruct C {\n    c: String,\n}\n#[derive(Deserialize)]\nstruct D {\n    d: String,\n}\n#[derive(other::Deserialize)]\nstruct E {\n    e: String,\n}\n";
const LOCAL_STRING: &str =
    "use serde::Deserialize;\nstruct String;\n#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n";
const ALIASED_STRING: &str =
    "use serde::Deserialize;\ntype Name = String;\n#[derive(Deserialize)]\nstruct A {\n    name: Name,\n}\n";
const ATTRIBUTED_FIELD: &str =
    "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A {\n    #[serde(default)]\n    name: String,\n}\n";
const SERDE_SETTINGS: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(transparent)]\nstruct A(String);\n#[derive(Deserialize)]\n#[serde(from = \"String\")]\nstruct B(String);\nimpl From<String> for B {\n    fn from(text: String) -> B { B(text) }\n}\n";

const TRY_FROM: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct A(String);\nimpl TryFrom<String> for A {\n    type Error = u8;\n    fn try_from(text: String) -> Result<A, u8> { Ok(A(text)) }\n}\n#[derive(Deserialize)]\n#[serde(deny_unknown_fields, try_from = \"String\")]\nstruct B(String);\nimpl std::convert::TryFrom<String> for B {\n    type Error = u8;\n    fn try_from(text: String) -> Result<B, u8> { Ok(B(text)) }\n}\n#[derive(Deserialize)]\n#[cfg_attr(feature = \"serde\", serde(try_from = \"String\"))]\nenum C {\n    Named(String),\n}\nimpl TryFrom<String> for C {\n    type Error = u8;\n    fn try_from(text: String) -> Result<C, u8> { Ok(C::Named(text)) }\n}\n#[derive(Deserialize)]\nstruct User {\n    a: A,\n    b: Option<B>,\n}\n";
const TRY_FROM_UNWRITTEN: &str =
    "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct A(String);\n";
const TRY_FROM_ELSEWHERE: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct A(String);\nstruct B;\nimpl TryFrom<String> for B {\n    type Error = u8;\n    fn try_from(text: String) -> Result<B, u8> { Ok(B) }\n}\n";

const TRY_FROM_UNASKED: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A(String);\nimpl TryFrom<String> for A {\n    type Error = u8;\n    fn try_from(text: String) -> Result<A, u8> { Ok(A(text)) }\n}\n";

const NEWTYPES: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct Id(String);\nstruct Plain(String);\nstruct Named {\n    text: String,\n}\nstruct Key(Plain);\nstruct C(B);\nstruct B(Z);\nstruct Z(String);\nmod inner {\n    pub struct Hidden(String);\n}\n#[derive(Deserialize)]\nstruct U {\n    id: Id,\n    plain: Plain,\n    named: Named,\n    nested: Vec<Option<Plain>>,\n    key: Key,\n    deep: C,\n    hidden: inner::Hidden,\n}\n";
const NOT_NEWTYPES: &str = "use serde::Deserialize;\nstruct Pair(String, String);\nstruct Unit;\nstruct Count(u32);\nstruct Two {\n    a: String,\n    b: String,\n}\n#[derive(Deserialize)]\nstruct U {\n    pair: Pair,\n    unit: Unit,\n    count: Count,\n    two: Two,\n}\n";
const ROOT: &str = "mod a;\nmod b;\nmod c;\n";
const PLAIN_ID: &str = "pub struct Id(String);\n";
const CHECKED_ID: &str = "#[derive(newtype::CheckedDeserialize)]\npub struct Id(String);\nimpl newtype::Checked for Id {\n    type Error = u8;\n    fn checked(text: &String) -> Result<&String, u8> { Ok(text) }\n}\n";
const USES_PLAIN: &str =
    "use serde::Deserialize;\nuse crate::a::Id;\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n";
const USES_CHECKED: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A {\n    id: crate::b::Id,\n}\n";

const CHECKED: &str = "use serde::Deserialize;\nuse newtype::{Checked, CheckedDeserialize};\n#[derive(newtype::Newtype, newtype::CheckedDeserialize)]\n#[newtype(checked)]\nstruct OneLine(String);\nimpl Checked for OneLine {\n    type Error = u8;\n    fn checked(text: &String) -> Result<&String, u8> { Ok(text) }\n}\n#[derive(CheckedDeserialize)]\nstruct Line(String);\nimpl newtype::Checked for Line {\n    type Error = u8;\n    fn checked(text: &String) -> Result<&String, u8> { Ok(text) }\n}\n#[derive(Deserialize)]\nstruct A {\n    line: OneLine,\n    lines: Vec<Line>,\n}\n";
const STRAY_CHECKED: &str = "use serde::Deserialize;\n#[derive(newtype::Newtype, newtype::Deserialize)]\npub struct Title(String);\npub struct Untitled;\nimpl newtype::Checked for Title {\n    type Error = Untitled;\n    fn checked(text: &String) -> Result<&String, Untitled> { Err(Untitled) }\n}\n#[derive(Deserialize)]\npub struct Entry {\n    title: Title,\n}\npub struct Line(String);\nimpl newtype::Checked for Line {\n    type Error = Untitled;\n    fn checked(text: &String) -> Result<&String, Untitled> { Err(Untitled) }\n}\n#[derive(Deserialize)]\npub struct Lines {\n    lines: Vec<Line>,\n}\n";
const UNCHECKED: &str = "use serde::Deserialize;\n#[derive(newtype::Newtype, newtype::Deserialize)]\nstruct Name(String);\n#[derive(Deserialize)]\nstruct A {\n    name: Name,\n}\n";
const OTHER_CHECKED: &str = "use serde::Deserialize;\nmod other {\n    pub use other_derive::CheckedDeserialize;\n}\n#[derive(other::CheckedDeserialize)]\nstruct Name(String);\nimpl newtype::Checked for Name {}\n#[derive(Deserialize)]\nstruct A {\n    name: Name,\n}\n";
const HAND_DESERIALIZE: &str = "use serde::Deserialize;\nstruct Id(String);\nimpl<'de> Deserialize<'de> for Id {\n    fn deserialize<D>(d: D) -> Result<Id, D::Error> {\n        let text = String::deserialize(d)?;\n        Id::from_str(&text).map_err(D::Error::custom)\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n";
const NEWTYPE_ROOT: &str =
    "mod boundary;\npub use boundary::Checked;\npub use newtype_derive::{CheckedDeserialize, Deserialize};\n";
const NEWTYPE_BOUNDARY: &str = "pub trait Checked {}\n";
const WORKSPACE_USER: &str = "use serde::Deserialize;\n#[derive(newtype::CheckedDeserialize)]\npub struct Line(String);\nimpl newtype::Checked for Line {}\n#[derive(newtype::Deserialize)]\npub struct Name(String);\nimpl newtype::Checked for Name {}\n#[derive(Deserialize)]\npub struct A {\n    line: Line,\n    name: Name,\n}\n";

const TESTED: &str = "#[cfg(test)]\nmod tests {\n    use serde::Deserialize;\n    #[derive(Deserialize)]\n    struct A {\n        name: String,\n    }\n}\n";
const INTEGRATION: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n";
const INLINE_MODULE: &str = "mod inner {\n    use serde::Deserialize;\n    #[derive(Deserialize)]\n    struct A {\n        name: String,\n    }\n    pub fn f(name: String) {}\n}\n";

const CASES: [Case; 24] = [
    Case::new(
        Title::new("a text leaf anywhere in a field's type"),
        &[Source::new(Located::new("a.rs"), Text::new(TEXT_FIELDS))],
        Flagged::new(&["a.rs:4", "a.rs:5", "a.rs:6", "a.rs:7", "a.rs:8", "a.rs:9", "a.rs:10", "a.rs:11"]),
    ),
    Case::new(
        Title::new("tuple fields and enum variant fields"),
        &[Source::new(Located::new("a.rs"), Text::new(TUPLE_AND_ENUM))],
        Flagged::new(&["a.rs:3", "a.rs:6", "a.rs:8"]),
    ),
    Case::new(
        Title::new("serde's derive by path, behind cfg_attr and among others"),
        &[Source::new(Located::new("a.rs"), Text::new(DERIVE_FORMS))],
        Flagged::new(&["a.rs:3", "a.rs:7", "a.rs:11"]),
    ),
    Case::new(
        Title::new("no derive that resolves to serde's or newtype's Deserialize"),
        &[Source::new(Located::new("a.rs"), Text::new(NOT_DESERIALIZED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a local type spelled String is not text"),
        &[Source::new(Located::new("a.rs"), Text::new(LOCAL_STRING))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an alias of String is text"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED_STRING))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("a field is flagged at its type, below its attributes"),
        &[Source::new(Located::new("a.rs"), Text::new(ATTRIBUTED_FIELD))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("transparent and from do not validate"),
        &[Source::new(Located::new("a.rs"), Text::new(SERDE_SETTINGS))],
        Flagged::new(&["a.rs:4", "a.rs:7"]),
    ),
    Case::new(
        Title::new("serde try_from with a written TryFrom impl validates"),
        &[Source::new(Located::new("a.rs"), Text::new(TRY_FROM))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("serde try_from without a written TryFrom impl"),
        &[Source::new(Located::new("a.rs"), Text::new(TRY_FROM_UNWRITTEN))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("serde try_from with a TryFrom impl for another type"),
        &[Source::new(Located::new("a.rs"), Text::new(TRY_FROM_ELSEWHERE))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a TryFrom impl without serde try_from"),
        &[Source::new(Located::new("a.rs"), Text::new(TRY_FROM_UNASKED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("an unvalidated newtype of text is text, through newtypes and containers"),
        &[Source::new(Located::new("a.rs"), Text::new(NEWTYPES))],
        Flagged::new(&["a.rs:3", "a.rs:17", "a.rs:18", "a.rs:19", "a.rs:20", "a.rs:21", "a.rs:22", "a.rs:23"]),
    ),
    Case::new(
        Title::new("pairs, units, numbers and two fields are not text newtypes"),
        &[Source::new(Located::new("a.rs"), Text::new(NOT_NEWTYPES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a newtype is the one its path resolves to, across files"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new(ROOT)),
            Source::new(Located::new("src/a.rs"), Text::new(PLAIN_ID)),
            Source::new(Located::new("src/b.rs"), Text::new(CHECKED_ID)),
            Source::new(Located::new("src/c.rs"), Text::new(USES_PLAIN)),
        ],
        Flagged::new(&["src/c.rs:5"]),
    ),
    Case::new(
        Title::new("a newtype validated in another file"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new(ROOT)),
            Source::new(Located::new("src/a.rs"), Text::new(PLAIN_ID)),
            Source::new(Located::new("src/b.rs"), Text::new(CHECKED_ID)),
            Source::new(Located::new("src/c.rs"), Text::new(USES_CHECKED)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("newtype's CheckedDeserialize validates its newtype and every user"),
        &[Source::new(Located::new("a.rs"), Text::new(CHECKED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an impl of newtype's Checked beside an unchecked derive, or none, does not validate"),
        &[Source::new(Located::new("a.rs"), Text::new(STRAY_CHECKED))],
        Flagged::new(&["a.rs:3", "a.rs:11", "a.rs:20"]),
    ),
    Case::new(
        Title::new("an unchecked newtype deriving newtype's Deserialize, and its user"),
        &[Source::new(Located::new("a.rs"), Text::new(UNCHECKED))],
        Flagged::new(&["a.rs:3", "a.rs:6"]),
    ),
    Case::new(
        Title::new("another derive named CheckedDeserialize, or a hand-written Deserialize, does not validate"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OTHER_CHECKED)),
            Source::new(Located::new("b.rs"), Text::new(HAND_DESERIALIZE)),
        ],
        Flagged::new(&["a.rs:10", "b.rs:11"]),
    ),
    Case::new(
        Title::new("newtype's derives declared in the workspace"),
        &[
            Source::new(Located::new("crates/newtype/src/lib.rs"), Text::new(NEWTYPE_ROOT)),
            Source::new(Located::new("crates/newtype/src/boundary.rs"), Text::new(NEWTYPE_BOUNDARY)),
            Source::new(Located::new("crates/user/src/lib.rs"), Text::new(WORKSPACE_USER)),
        ],
        Flagged::new(&["crates/user/src/lib.rs:6", "crates/user/src/lib.rs:11"]),
    ),
    Case::new(
        Title::new("a test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TESTED))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("an integration test"),
        &[Source::new(Located::new("x/tests/a.rs"), Text::new(INTEGRATION))],
        Flagged::new(&["x/tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("an inline module, beside a function taking text"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_MODULE))],
        Flagged::new(&["a.rs:5"]),
    ),
];

#[test]
fn deserialized_data_holds_no_free_text() {
    assert_eq!(Sources::mismatches::<Queried<NoPlaintext>>(&CASES), Lines::default());
}
