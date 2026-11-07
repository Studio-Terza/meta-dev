use grammar::artifact::Artifacts;
use grammar::cited::Cited;
use grammar::flag::Flags;
use grammar::ident::Words;
use grammar::line::Unbroken;
use grammar::message::{Depth, Lexemes, Mark, Number};
use grammar::node::{Ids, Spelled};
use grammar::output::field::{Bare, Id, Integer, Quoted, Ref};
use grammar::output::{Fields, Unread};
use grammar::search::Term;
use grammar::search::Terms;
use grammar::slug::Letters;
use laws::proptest::prelude::prop_assert_eq;

struct Marks;

impl Marks {
    fn of(text: &str) -> Vec<Mark> {
        Lexemes::scanned(&text).into_iter().map(|lexeme| lexeme.mark()).filter(|mark| *mark != Mark::Plain).collect()
    }
}

laws::table!(
    markers_are_recognised_at_line_starts,
    Marks::of,
    [
        "1. one\n2) two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "1. one\n  2. sub" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Nested)],
        "**1. one**\n**2.** two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "- a\n* b\n> c" => vec![Mark::Bulleted, Mark::Bulleted, Mark::Quoted],
        "yes. no? ok" => vec![Mark::Stop, Mark::Stop],
        "v0.2.1 is fine" => Vec::<Mark>::new(),
        "see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b now" => vec![Mark::Uuid],
        "line\nnext" => vec![Mark::Break],
    ]
);

laws::law!(lexemes_tile_the_text_without_gaps, "[ -~\n\t•é]{0,60}", text, {
    let rebuilt: String = Lexemes::scanned(&text).iter().map(|lexeme| lexeme.span().slice(&text)).collect();
    prop_assert_eq!(rebuilt, text);
});

laws::table!(
    slugs_keep_letters_and_digits_and_replace_every_other_character,
    Letters::slugged,
    [
        "/Users/me/my_repo" => "-Users-me-my-repo",
        "abc123" => "abc123",
        "a.b é" => "a-b--",
        "" => "",
    ]
);

laws::table!(
    camel_names_split_into_lowered_humps,
    Words::lowered_humps,
    [
        "Transcripts" => vec!["transcripts"],
        "ReadFrom" => vec!["read", "from"],
        "lowerThenUpper" => vec!["lower", "then", "upper"],
        "" => Vec::<String>::new(),
    ]
);

laws::table!(
    snake_names_raise_into_camel_case,
    Words::camel,
    [
        "read_from" => "ReadFrom",
        "sessions" => "Sessions",
        "a__b_" => "AB",
        "" => "",
    ]
);

struct Lexed;

impl Lexed {
    fn terms(words: &[&str]) -> Vec<String> {
        Terms::lexed(words.iter().copied())
            .map(|terms| [terms.first()].into_iter().chain(terms.rest()).map(ToString::to_string).collect())
            .unwrap_or_default()
    }
}

laws::table!(
    search_words_lex_into_lowered_terms_split_at_every_other_character,
    Lexed::terms,
    [
        ["Pinning-tests", "NOW"] => vec!["pinning", "tests", "now"],
        ["\"a OR b*\"", "c:d"] => vec!["a", "or", "b", "c", "d"],
        ["Écoute", "v0.2"] => vec!["écoute", "v0", "2"],
        ["--", "..."] => Vec::<String>::new(),
    ]
);

laws::table!(
    node_ids_are_n_and_a_number_without_leading_zeros_that_fits,
    Ids::spelled,
    [
        "n0" => Spelled::Numbered(0),
        "n42" => Spelled::Numbered(42),
        "n9223372036854775807" => Spelled::Numbered(i64::MAX),
        "n00" => Spelled::Misnumbered,
        "n001" => Spelled::Misnumbered,
        "n9223372036854775808" => Spelled::Misnumbered,
        "n99999999999999999999" => Spelled::Misnumbered,
        "n" => Spelled::Unspelled,
        "42" => Spelled::Unspelled,
        "n42x" => Spelled::Unspelled,
    ]
);

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

laws::table!(
    one_unbroken_line_with_a_visible_character_is_spelled_whole,
    |text: &&str| Unbroken::spelled(text).map(str::to_owned),
    [
        "a condition" => Some("a condition".to_owned()),
        "  padded " => Some("  padded ".to_owned()),
        "\tx" => Some("\tx".to_owned()),
        "" => None,
        "  " => None,
        "\t" => None,
        "a\nb" => None,
        "a\r\n" => None,
        "\na" => None,
        "\u{b}" => None,
        "\u{c}" => None,
        "\u{a0}" => None,
        "\u{2028}" => None,
        "\u{85}" => None,
        "\u{3000}\t" => None,
        "\u{a0}x" => Some("\u{a0}x".to_owned()),
        "x\u{b}y" => None,
        "x\u{c}y" => None,
        "x\u{85}y" => None,
        "x\u{2028}y" => None,
        "x\u{2029}y" => None,
        "x\ry" => None,
        "x\u{2029}" => None,
        "\u{85}x" => None,
        "x\u{2007}y" => Some("x\u{2007}y".to_owned()),
    ]
);

laws::table!(
    a_word_led_by_two_dashes_is_spelled_as_a_flag,
    |text: &&str| Flags::spelled(text).map(str::to_owned),
    [
        "--reason" => Some("--reason".to_owned()),
        "--" => Some("--".to_owned()),
        "---x" => Some("---x".to_owned()),
        "--no such" => Some("--no such".to_owned()),
        "-r" => None,
        "demo" => None,
        "a--b" => None,
        " --reason" => None,
        "" => None,
    ]
);

struct Read;

#[derive(Debug, PartialEq, Eq)]
struct Read6 {
    node: i64,
    quoted: Term,
    bare: Term,
    integer: u64,
    ordinal: usize,
    message: Term,
}

impl Read {
    fn quoted(text: &str) -> Option<Term> {
        let mut fields = Fields::lexed(&Fields::quoting(text)).ok()?;
        let read = fields.field::<Quoted<Term>>().ok()?;
        fields.ended(read).ok()
    }

    fn line(text: &&str) -> Option<Read6> {
        let mut fields = Fields::lexed(text).ok()?;
        let line = Read::fields(&mut fields).ok()?;
        fields.ended(line).ok()
    }

    fn fields(fields: &mut Fields) -> Result<Read6, Unread> {
        Ok(Read6 {
            node: fields.field::<Id<Node>>()?.number(),
            quoted: fields.field::<Quoted<Term>>()?,
            bare: fields.field::<Bare<Term>>()?,
            integer: fields.past(&["to".to_owned()])?.field::<Integer<u64>>()?,
            ordinal: fields.field::<Cited>()?.ordinal(),
            message: fields.field::<Ref<Term>>()?,
        })
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

laws::roundtrip!(
    any_text_is_quoted_and_lexed_back_as_one_quoted_field,
    "(?s).{0,40}",
    |text: &String| text.clone(),
    |text: &String| Read::quoted(text),
    |text: String| Some(Term::new(text))
);

laws::table!(
    an_output_line_lexes_into_words_refs_ids_quotes_and_numbers,
    Read::line,
    [
        "n4 \"de\\\"mo\" feature to 12 deadbeef.3 85b287b3-4d5e-4f60\n" => Some(Read6 {
            node: 4,
            quoted: Term::new("de\"mo".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 12,
            ordinal: 3,
            message: Term::new("85b287b3-4d5e-4f60".to_owned()),
        }),
        "n4 \"x\" feature to 12 deadbeef.3 deadbeef extra" => None,
        "n4 \"x\" feature from 12 deadbeef.3 deadbeef" => None,
        "n4 \"x\" ( feature" => None,
    ]
);
