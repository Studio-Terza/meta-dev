#![deny(clippy::wildcard_enum_match_arm)]

use grammar_artifact::Artifacts;

struct Spellings;

impl Spellings {
    fn artifact(text: &str) -> Option<[String; 2]> {
        Artifacts::spelled(text).map(|spelled| spelled.joined(|kind, name| [kind.to_owned(), name.to_owned()]))
    }
}

laws::table!(
    an_artifact_is_spelled_as_a_lowercase_kind_a_colon_and_a_name,
    |text: &&str| Spellings::artifact(text),
    [
        "lint:no-comments" => Some(["lint".to_owned(), "no-comments".to_owned()]),
        "behaviour:tests::a_law" => Some(["behaviour".to_owned(), "tests::a_law".to_owned()]),
        "tool:crates/design.rs" => Some(["tool".to_owned(), "crates/design.rs".to_owned()]),
        "Lint:x" => None,
        "lint" => None,
        "lint:" => None,
        ":x" => None,
        "lint:x y" => None,
        "lint :x" => None,
        "lint: x" => None,
        "lint:-x" => None,
        "" => None,
    ]
);
