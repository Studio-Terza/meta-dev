#![deny(clippy::wildcard_enum_match_arm)]

use grammar_cited::Cited;
use grammar_output::field::{Bare, Id, Integer, Quoted};
use grammar_output::{Fields, Unread};
use grammar_search::Term;

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
            message: fields.field::<Bare<Term>>()?,
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
        "n4 \"x\" feature to 1234567 1a2b3c4.0 0a1b2c3" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 1_234_567,
            ordinal: 0,
            message: Term::new("0a1b2c3".to_owned()),
        }),
        "n4 \"x\" feature to 7 9abcdef0.12 7e57ab1-0000" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 7,
            ordinal: 12,
            message: Term::new("7e57ab1-0000".to_owned()),
        }),
        "n4 \"x\" feature to 7 o1.0 O1" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 7,
            ordinal: 0,
            message: Term::new("O1".to_owned()),
        }),
        "n4 \"x\" feature to 7 o1. o1" => None,
        "n4 \"x\" feature to 7 o1.01 o1" => None,
        "n4 \"x\" feature to 1a2b3c4 1a2b3c4.0 1a2b3c4" => None,
        "n4 \"x\" feature to 12 deadbeef.3 deadbeef extra" => None,
        "n4 \"x\" feature from 12 deadbeef.3 deadbeef" => None,
        "n4 \"x\" ( feature" => None,
    ]
);
