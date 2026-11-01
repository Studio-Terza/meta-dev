use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use lint_no_plaintext::NoPlaintext;

const CASES: [Case; 53] = [
    Case::new(
        Title::new("string field"),
        &[Source::new(Located::new("a.rs"), Text::new("#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n"))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("str reference field"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A<'a> {\n    name: &'a str,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("cow of str field"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A<'a> {\n    name: Cow<'a, str>,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("boxed str field"),
        &[Source::new(Located::new("a.rs"), Text::new("#[derive(Deserialize)]\nstruct A {\n    name: Box<str>,\n}\n"))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("option of string"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    name: Option<String>,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("vec of string"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    names: Vec<String>,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("map keyed by string"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    names: BTreeMap<String, u8>,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("path to string"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    name: std::string::String,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("tuple struct string"),
        &[Source::new(Located::new("a.rs"), Text::new("#[derive(Deserialize)]\nstruct A(String);\n"))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("each text field"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    name: String,\n    count: u32,\n    note: String,\n}\n"),
        )],
        Flagged::new(&["a.rs:3", "a.rs:5"]),
    ),
    Case::new(
        Title::new("enum variant fields"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\nenum A {\n    Named(String),\n    Counted(u32),\n    Noted { note: String },\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:3", "a.rs:5"]),
    ),
    Case::new(
        Title::new("serde path derive"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(serde::Deserialize)]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("derive behind cfg_attr"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[cfg_attr(feature = \"serde\", derive(Deserialize))]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("derive among others"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Debug, Deserialize, Clone)]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("transparent does not validate"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\n#[serde(transparent)]\nstruct A(String);\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("from does not validate"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\n#[serde(from = \"String\")]\nstruct A(String);\n"),
        )],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("without Deserialize"),
        &[Source::new(Located::new("a.rs"), Text::new("#[derive(Debug)]\nstruct A {\n    name: String,\n}\n"))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("Serialize only"),
        &[Source::new(Located::new("a.rs"), Text::new("#[derive(Serialize)]\nstruct A {\n    name: String,\n}\n"))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("cfg_attr without Deserialize"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[cfg_attr(feature = \"serde\", derive(Serialize))]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("typed fields"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\nstruct A {\n    count: u32,\n    path: PathBuf,\n    when: Option<Duration>,\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("other path ending in str"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    name: other::Strand,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("try_from on the type"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct A(String);\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("try_from among settings"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\n#[serde(deny_unknown_fields, try_from = \"String\")]\nstruct A(String);\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("try_from behind cfg_attr"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\n#[cfg_attr(feature = \"serde\", serde(try_from = \"String\"))]\nstruct A(String);\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("try_from on an enum"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nenum A {\n    Named(String),\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("newtype without validation"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\nstruct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:2", "a.rs:5"]),
    ),
    Case::new(
        Title::new("newtype without Deserialize"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n"),
        )],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("named single field wrapper"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Id {\n    text: String,\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n"),
        )],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("newtype in another file"),
        &[
            Source::new(Located::new("a.rs"), Text::new("struct Id(String);\n")),
            Source::new(Located::new("b.rs"), Text::new("#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n")),
        ],
        Flagged::new(&["b.rs:3"]),
    ),
    Case::new(
        Title::new("newtype inside another type"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    ids: Vec<Option<Id>>,\n}\n"),
        )],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("newtype of a newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Id(String);\nstruct Key(Id);\n#[derive(Deserialize)]\nstruct A {\n    key: Key,\n}\n"),
        )],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("newtype of a newtype declared first"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Key(Id);\nstruct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    key: Key,\n}\n"),
        )],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("three newtypes deep"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct C(B);\nstruct B(A);\nstruct A(String);\n#[derive(Deserialize)]\nstruct U {\n    c: C,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("newtype in an inline module"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "mod inner {\n    pub struct Id(String);\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("validating newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[derive(Deserialize)]\n#[serde(try_from = \"String\")]\nstruct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("validating newtype in another file"),
        &[
            Source::new(Located::new("a.rs"), Text::new("#[serde(try_from = \"String\")]\nstruct Id(String);\n")),
            Source::new(Located::new("b.rs"), Text::new("#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n")),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("newtype of a validating newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[serde(try_from = \"String\")]\nstruct Id(String);\nstruct Key(Id);\n#[derive(Deserialize)]\nstruct A {\n    key: Key,\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("hand-written Deserialize with from_str"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nimpl<'de> Deserialize<'de> for Id {\n    fn deserialize<D>(d: D) -> Result<Id, D::Error> {\n        let text = String::deserialize(d)?;\n        Id::from_str(&text).map_err(D::Error::custom)\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("hand-written Deserialize with parse"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nimpl<'de> serde::Deserialize<'de> for Id {\n    fn deserialize<D>(d: D) -> Result<Id, D::Error> {\n        let text = String::deserialize(d)?;\n        text.parse().map_err(D::Error::custom)\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("hand-written Deserialize without FromStr"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nimpl<'de> Deserialize<'de> for Id {\n    fn deserialize<D>(d: D) -> Result<Id, D::Error> {\n        String::deserialize(d).map(Id)\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:9"]),
    ),
    Case::new(
        Title::new("FromStr impl without hand-written Deserialize"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nimpl FromStr for Id {\n    type Err = E;\n    fn from_str(text: &str) -> Result<Id, E> {\n        Ok(Id(text.to_owned()))\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("hand-written Deserialize for another type"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nstruct Other(u8);\nimpl<'de> Deserialize<'de> for Other {\n    fn deserialize<D>(d: D) -> Result<Other, D::Error> {\n        Other::from_str(\"\")\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("hand-written impl of another trait"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "struct Id(String);\nimpl Tr for Id {\n    fn f() {\n        Id::from_str(\"\");\n    }\n}\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n",
            ),
        )],
        Flagged::new(&["a.rs:9"]),
    ),
    Case::new(
        Title::new("two fields are not a newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Pair(String, String);\n#[derive(Deserialize)]\nstruct A {\n    pair: Pair,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("unit struct is not a newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Unit;\n#[derive(Deserialize)]\nstruct A {\n    unit: Unit,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("newtype of a number"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("struct Count(u32);\n#[derive(Deserialize)]\nstruct A {\n    count: Count,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("unknown type"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    id: elsewhere::Id,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new(
                "#[cfg(test)]\nmod tests {\n    #[derive(Deserialize)]\n    struct A {\n        name: String,\n    }\n}\n",
            ),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test only type"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[cfg(test)]\n#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test only newtype"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("#[cfg(test)]\nstruct Id(String);\n#[derive(Deserialize)]\nstruct A {\n    id: Id,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(
            Located::new("a.rs"),
            Text::new("mod inner {\n    #[derive(Deserialize)]\n    struct A {\n        name: String,\n    }\n}\n"),
        )],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("integration test"),
        &[Source::new(
            Located::new("x/tests/a.rs"),
            Text::new("#[derive(Deserialize)]\nstruct A {\n    name: String,\n}\n"),
        )],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("unrelated module"),
        &[Source::new(Located::new("a.rs"), Text::new("mod inner {\n    pub fn f(name: String) {}\n}\n"))],
        Flagged::new(&[]),
    ),
];

#[test]
fn deserialized_data_holds_no_free_text() {
    assert_eq!(Sources::mismatches::<NoPlaintext>(&CASES), Lines::default());
}
