use std::path::PathBuf;

use file_ports::{Bytes, Memory};
use lint_core::SourceFile;
use lint_fixtures::{Case, Checks, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_layout::Failure;
use lint_no_plaintext::NoPlaintext;
use lint_placement::Placement;

struct Manifest {
    path: &'static str,
    text: &'static str,
}

const MANIFESTS: [Manifest; 11] = [
    Manifest {
        path: "Cargo.toml",
        text: "[package]\nname = \"fixture\"\n[dependencies]\ndiesel = \"2\"\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "a/Cargo.toml",
        text: "[package]\nname = \"a\"\n[dependencies]\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "b/Cargo.toml",
        text: "[package]\nname = \"b\"\n[dependencies]\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "x/Cargo.toml",
        text: "[package]\nname = \"x\"\n[dependencies]\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "crates/newtype/Cargo.toml",
        text: "[package]\nname = \"newtype\"\n[dependencies]\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "crates/user/Cargo.toml",
        text: "[package]\nname = \"user\"\n[dependencies]\nserde = \"1\"\nserde_derive = \"1\"\nnewtype = \"1\"\nthiserror = \"1\"\n",
    },
    Manifest {
        path: "renamed/Cargo.toml",
        text: "[package]\nname = \"renamed\"\n[dependencies]\nwire = { package = \"serde\", version = \"1\" }\nserde = { package = \"not-serde\", version = \"1\" }\n",
    },
    Manifest { path: "undeclared/Cargo.toml", text: "[package]\nname = \"undeclared\"\n" },
    Manifest {
        path: "ws/Cargo.toml",
        text: "[workspace]\nmembers = [\"member\"]\n[workspace.dependencies]\nwire = { package = \"serde\", version = \"1\" }\n",
    },
    Manifest {
        path: "ws/member/Cargo.toml",
        text: "[package]\nname = \"member\"\n[dependencies]\nwire.workspace = true\n",
    },
    Manifest {
        path: "platform/Cargo.toml",
        text: "[package]\nname = \"platform\"\n[target.'cfg(unix)'.dependencies]\nwire = { package = \"serde_derive\", version = \"1\" }\n",
    },
];
const RENAMED: &str = "#[derive(wire::Deserialize)]\nstruct A {\n    a: String,\n}\n#[derive(serde::Deserialize)]\nstruct B {\n    b: String,\n}\n";
const UNDECLARED: &str = "#[derive(serde::Deserialize)]\nstruct A {\n    a: String,\n}\n";
const INHERITED: &str = "#[derive(wire::Deserialize)]\nstruct A {\n    a: String,\n}\n";

const TEXT_FIELDS: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A<'a> {\n    owned: String,\n    borrowed: &'a str,\n    cow: Cow<'a, str>,\n    boxed: Box<str>,\n    optional: Option<String>,\n    many: Vec<String>,\n    keyed: BTreeMap<String, u8>,\n    pathed: std::string::String,\n    count: u32,\n    path: std::path::PathBuf,\n    other: other::Strand,\n    elsewhere: elsewhere::Id,\n}\n";
const TUPLE_AND_ENUM: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A(String);\n#[derive(Deserialize)]\nenum B {\n    Named(String),\n    Counted(u32),\n    Noted { note: String },\n}\n";
const DERIVE_FORMS: &str = "#[derive(serde::Deserialize)]\nstruct A {\n    a: String,\n}\n#[cfg_attr(feature = \"serde\", derive(serde::Deserialize))]\nstruct B {\n    b: String,\n}\n#[derive(Debug, serde::Deserialize, Clone)]\nstruct C {\n    c: String,\n}\n";
const NOT_DESERIALIZED: &str = "use serde::Serialize;\n#[derive(Debug)]\nstruct A {\n    a: String,\n}\n#[derive(Serialize)]\nstruct B {\n    b: String,\n}\n#[cfg_attr(feature = \"serde\", derive(Serialize))]\nstruct C {\n    c: String,\n}\n#[derive(other::Deserialize)]\nstruct D {\n    d: String,\n}\n#[derive(Debug, Clone, PartialEq, thiserror::Error, newtype::Newtype)]\nstruct E {\n    e: String,\n}\n";
const DERIVE_CRATE: &str = "#[derive(serde_derive::Deserialize)]\nstruct A {\n    a: String,\n}\n";
const UNRESOLVED_DERIVES: &str = "mod globbed {\n    use serde::*;\n    #[derive(Deserialize)]\n    struct A {\n        a: ::std::string::String,\n    }\n}\n#[derive(Deserialize)]\nstruct C {\n    c: String,\n}\nmod failed {\n    use other::*;\n    #[derive(Debug, Error)]\n    pub struct Failed {\n        message: ::std::string::String,\n    }\n}\n#[derive(Unimported)]\nstruct D {\n    count: u32,\n}\n";
const SUBMODULE_DERIVES: &str = "use diesel::prelude::Queryable;\nuse serde::Deserialize;\n#[derive(Queryable)]\npub struct Label {\n    text: String,\n}\n#[derive(diesel::prelude::Insertable, newtype::serde::Deserialize)]\npub struct Note {\n    text: String,\n}\n#[derive(Deserialize)]\npub struct Entry {\n    text: String,\n}\n";
const RESOLVED_OTHER_DERIVE: &str =
    "use thiserror::Error;\n#[derive(Debug, Error)]\npub struct Failed {\n    message: String,\n}\n";
const GLOBBED_FIELD_TYPE: &str = "use chrono::*;\nuse serde::Deserialize;\n#[derive(Deserialize)]\npub struct Stamp {\n    at: NaiveDate,\n    note: ::std::string::String,\n}\n";
const UNKNOWN_FIELD_TYPE: &str = "#[derive(serde::Deserialize)]\nstruct A {\n    id: Id,\n}\n#[derive(Debug)]\nstruct B {\n    id: Id,\n}\n#[derive(Unimported)]\nstruct C {\n    id: Id,\n}\n";
const REEXPORTED_ROOT: &str = "pub use serde;\npub mod de {\n    pub use serde::Deserialize;\n}\n";
const REEXPORTED_USER: &str = "#[derive(newtype::serde::Deserialize)]\npub struct A {\n    a: String,\n}\n#[derive(newtype::de::Deserialize)]\npub struct B {\n    b: String,\n}\n";
const LOCAL_STRING: &str =
    "use serde::Deserialize;\nstruct String;\n#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n";
const ALIASED_STRING: &str =
    "use serde::Deserialize;\ntype Name = String;\n#[derive(Deserialize)]\nstruct A {\n    name: Name,\n}\n";
const ATTRIBUTED_FIELD: &str =
    "use serde::Deserialize;\n#[derive(Deserialize)]\nstruct A {\n    #[serde(default)]\n    name: String,\n}\n";
const SERDE_SETTINGS: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(transparent)]\nstruct A(String);\n#[derive(Deserialize)]\n#[serde(from = \"String\")]\nstruct B(String);\nimpl From<String> for B {\n    fn from(text: String) -> B { B(text) }\n}\n";

const PARAMETER_BOUND: &str =
    "#[derive(serde::Deserialize)]\nstruct A<T: From<String>> {\n    t: T,\n}\ntype Alias = String;\n";
const TRY_FROM: &str = "use serde::Deserialize;\n#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct A(String);\nimpl TryFrom<String> for A {\n    type Error = u8;\n    fn try_from(text: String) -> Result<A, u8> { Ok(A(text)) }\n}\n#[derive(Deserialize)]\n#[serde(deny_unknown_fields, try_from = \"String\")]\nstruct B(String);\nimpl std::convert::TryFrom<String> for B {\n    type Error = u8;\n    fn try_from(text: String) -> Result<B, u8> { Ok(B(text)) }\n}\n#[derive(Deserialize)]\n#[cfg_attr(feature = \"serde\", serde(try_from = \"String\"))]\nenum C {\n    Named(String),\n}\nimpl TryFrom<String> for C {\n    type Error = u8;\n    fn try_from(text: String) -> Result<C, u8> { Ok(C::Named(text)) }\n}\n#[derive(Deserialize)]\nstruct User {\n    a: A,\n    b: Option<B>,\n}\n";
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

const CASES: [Case; 33] = [
    Case::new(
        Title::new("a text leaf anywhere in a field's type"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(TEXT_FIELDS))],
        Flagged::new(&[
            "src/lib.rs:4",
            "src/lib.rs:5",
            "src/lib.rs:6",
            "src/lib.rs:7",
            "src/lib.rs:8",
            "src/lib.rs:9",
            "src/lib.rs:10",
            "src/lib.rs:11",
        ]),
    ),
    Case::new(
        Title::new("tuple fields and enum variant fields"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(TUPLE_AND_ENUM))],
        Flagged::new(&["src/lib.rs:3", "src/lib.rs:6", "src/lib.rs:8"]),
    ),
    Case::new(
        Title::new("serde's derive by path, behind cfg_attr and among others"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(DERIVE_FORMS))],
        Flagged::new(&["src/lib.rs:3", "src/lib.rs:7", "src/lib.rs:11"]),
    ),
    Case::new(
        Title::new("a derive that resolves elsewhere is not serde's Deserialize"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(NOT_DESERIALIZED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("serde's derive through its derive crate"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(DERIVE_CRATE))],
        Flagged::new(&["src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("a derive through a glob or no import is not resolvable, at the derive"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(UNRESOLVED_DERIVES))],
        Flagged::new(&["src/lib.rs:3", "src/lib.rs:8", "src/lib.rs:14"]),
    ),
    Case::new(
        Title::new("a derive from an external crate's submodule is that crate's, not serde's"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(SUBMODULE_DERIVES))],
        Flagged::new(&["src/lib.rs:13"]),
    ),
    Case::new(
        Title::new("a resolved derive that is not serde's"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(RESOLVED_OTHER_DERIVE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an unresolvable field type is not text, under any derive"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(UNKNOWN_FIELD_TYPE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a field type reached through an external glob is not text, a String beside it is"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(GLOBBED_FIELD_TYPE))],
        Flagged::new(&["src/lib.rs:6"]),
    ),
    Case::new(
        Title::new("serde's derive through a workspace crate's re-exports"),
        &[
            Source::new(Located::new("crates/newtype/src/lib.rs"), Text::new(REEXPORTED_ROOT)),
            Source::new(Located::new("crates/user/src/lib.rs"), Text::new(REEXPORTED_USER)),
        ],
        Flagged::new(&["crates/user/src/lib.rs:3", "crates/user/src/lib.rs:7"]),
    ),
    Case::new(
        Title::new("a local type spelled String is not text"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(LOCAL_STRING))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an alias of String is text"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(ALIASED_STRING))],
        Flagged::new(&["src/lib.rs:5"]),
    ),
    Case::new(
        Title::new("a field is flagged at its type, below its attributes"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(ATTRIBUTED_FIELD))],
        Flagged::new(&["src/lib.rs:5"]),
    ),
    Case::new(
        Title::new("text in a type parameter's bound or an alias is not a field"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(PARAMETER_BOUND))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("transparent and from do not validate"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(SERDE_SETTINGS))],
        Flagged::new(&["src/lib.rs:4", "src/lib.rs:7"]),
    ),
    Case::new(
        Title::new("serde try_from with a written TryFrom impl does not validate"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(TRY_FROM))],
        Flagged::new(&["src/lib.rs:4", "src/lib.rs:11", "src/lib.rs:19", "src/lib.rs:27", "src/lib.rs:28"]),
    ),
    Case::new(
        Title::new("an unvalidated newtype of text is text, through newtypes and containers"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(NEWTYPES))],
        Flagged::new(&[
            "src/lib.rs:3",
            "src/lib.rs:17",
            "src/lib.rs:18",
            "src/lib.rs:19",
            "src/lib.rs:20",
            "src/lib.rs:21",
            "src/lib.rs:22",
            "src/lib.rs:23",
        ]),
    ),
    Case::new(
        Title::new("pairs, units, numbers and two fields are not text newtypes"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(NOT_NEWTYPES))],
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
        &[Source::new(Located::new("src/lib.rs"), Text::new(CHECKED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an impl of newtype's Checked beside an unchecked derive, or none, does not validate"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(STRAY_CHECKED))],
        Flagged::new(&["src/lib.rs:3", "src/lib.rs:11", "src/lib.rs:20"]),
    ),
    Case::new(
        Title::new("an unchecked newtype deriving newtype's Deserialize, and its user"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(UNCHECKED))],
        Flagged::new(&["src/lib.rs:3", "src/lib.rs:6"]),
    ),
    Case::new(
        Title::new("another derive named CheckedDeserialize, or a hand-written Deserialize, does not validate"),
        &[
            Source::new(Located::new("a/src/lib.rs"), Text::new(OTHER_CHECKED)),
            Source::new(Located::new("b/src/lib.rs"), Text::new(HAND_DESERIALIZE)),
        ],
        Flagged::new(&["a/src/lib.rs:10", "b/src/lib.rs:11"]),
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
        &[Source::new(Located::new("src/lib.rs"), Text::new(TESTED))],
        Flagged::new(&["src/lib.rs:6"]),
    ),
    Case::new(
        Title::new("an integration test"),
        &[Source::new(Located::new("x/tests/a.rs"), Text::new(INTEGRATION))],
        Flagged::new(&["x/tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("an inline module, beside a function taking text"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(INLINE_MODULE))],
        Flagged::new(&["src/lib.rs:5"]),
    ),
    Case::new(
        Title::new("serde's derive is the one its dependency's package names, renamed or not"),
        &[Source::new(Located::new("renamed/src/lib.rs"), Text::new(RENAMED))],
        Flagged::new(&["renamed/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("a crate path no dependency declares is not serde's"),
        &[Source::new(Located::new("undeclared/src/lib.rs"), Text::new(UNDECLARED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a dependency inherited from the workspace names its package there"),
        &[Source::new(Located::new("ws/member/src/lib.rs"), Text::new(INHERITED))],
        Flagged::new(&["ws/member/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("a platform's dependency names its package"),
        &[Source::new(Located::new("platform/src/lib.rs"), Text::new(INHERITED))],
        Flagged::new(&["platform/src/lib.rs:3"]),
    ),
];

struct Packaged;

impl Packaged {
    fn manifests() -> Memory {
        MANIFESTS.iter().map(|manifest| (PathBuf::from(manifest.path), Bytes::from(manifest.text))).collect()
    }

    fn parsed(manifests: &Memory, source: &Source) -> SourceFile {
        let shown = Sources::file(source.path(), source.text());
        let path = source.path().to_path_buf();
        let placement = Placement::locate(manifests, &path);
        Self::read(SourceFile::parse_at(path.clone(), path, placement, shown.text().clone()))
    }

    fn read(parsed: Result<SourceFile, Failure>) -> SourceFile {
        match parsed {
            Ok(file) => file,
            Err(failure) => unreachable!("fixture source parses: {failure}"),
        }
    }

    fn flagged(cases: &[Case]) -> Vec<String> {
        let manifests = Self::manifests();
        cases
            .iter()
            .map(|case| {
                let files: Vec<SourceFile> =
                    case.sources().iter().map(|source| Self::parsed(&manifests, source)).collect();
                format!("{} flagged {}", case.name(), Queried::<NoPlaintext>::flagged(&files))
            })
            .collect()
    }

    fn wanted(cases: &[Case]) -> Vec<String> {
        cases.iter().map(|case| format!("{} flagged {}", case.name(), Lines::from(case.flagged()))).collect()
    }
}

#[test]
fn deserialized_data_holds_no_free_text() {
    assert_eq!(Packaged::flagged(&CASES), Packaged::wanted(&CASES));
}
