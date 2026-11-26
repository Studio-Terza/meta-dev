#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_output::field::{Id, Integer, Phrase, Word};
use grammar_output::{Fields, Record};
use proptest::prelude::{prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

struct Read;

#[derive(Debug, PartialEq, Eq, Record)]
struct Read5 {
    #[field(Id<Node>, via = Node::number)]
    node: i64,
    #[field(Phrase<String>)]
    quoted: String,
    #[field(Word<String>)]
    bare: String,
    #[past("to".to_owned())]
    #[field(Integer<u64>)]
    integer: u64,
    #[field(Word<String>)]
    message: String,
}

impl Read {
    fn quoted(text: &str) -> Option<String> {
        Fields::lexed(&Fields::quoting(text))
            .ignoring(Unlexed::NotThisGrammar)?
            .field::<Phrase<String>>()
            .ignoring(Unlexed::NotThisGrammar)?
            .ended()
            .ignoring(Unlexed::NotThisGrammar)
    }

    fn line(text: &str) -> Option<Read5> {
        Fields::lexed(text)
            .ignoring(Unlexed::NotThisGrammar)?
            .record::<Read5>()
            .ignoring(Unlexed::NotThisGrammar)?
            .ended()
            .ignoring(Unlexed::NotThisGrammar)
    }

    fn of(quoted: &str, integer: u64, message: &str) -> Read5 {
        Read5 { node: 4, quoted: quoted.to_owned(), bare: "feature".to_owned(), integer, message: message.to_owned() }
    }
}

struct Node(i64);

impl std::str::FromStr for Node {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Node, Self::Err> {
        text.get(1..).unwrap_or_default().parse().map(Node)
    }
}

impl Node {
    fn number(self) -> i64 {
        self.0
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn any_text_is_quoted_and_lexed_back_as_one_quoted_field(text in "(?s).{0,40}") {
        prop_assert_eq!(Read::quoted(&text), Some(text));
    }
}

#[test]
fn an_output_line_lexes_into_words_ids_quotes_and_numbers() {
    assert_eq!(
        Read::line("n4 \"de\\\"mo\" feature to 12 85b287b3-4d5e-4f60\n"),
        Some(Read::of("de\"mo", 12, "85b287b3-4d5e-4f60"))
    );
    assert_eq!(Read::line("n4 \"x\" feature to 1234567 0a1b2c3"), Some(Read::of("x", 1_234_567, "0a1b2c3")));
    assert_eq!(Read::line("n4 \"x\" feature to 7 O1"), Some(Read::of("x", 7, "O1")));
    assert_eq!(Read::line("n4 \"x\" feature to 1a2b3c4 1a2b3c4"), None);
    assert_eq!(Read::line("n4 \"x\" feature to 12 deadbeef extra"), None);
    assert_eq!(Read::line("n4 \"x\" feature from 12 deadbeef"), None);
    assert_eq!(Read::line("n4 \"x\" ( feature"), None);
}
