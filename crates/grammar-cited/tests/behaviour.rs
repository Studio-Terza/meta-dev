#![deny(clippy::wildcard_enum_match_arm)]

use grammar_cited::{Cited, Uncited};

struct Citations;

#[derive(Debug, PartialEq, Eq)]
struct Citation {
    prefix: String,
    ordinal: u32,
}

impl Citations {
    fn read(text: &&str) -> Result<Citation, Uncited> {
        text.parse::<Cited>().map(|cited| Citation { prefix: cited.prefix().to_owned(), ordinal: cited.ordinal() })
    }
}

laws::table!(
    a_citation_reads_as_a_prefix_and_an_ordinal,
    Citations::read,
    [
        "a.3" => Ok(Citation { prefix: String::from("a"), ordinal: 3 }),
        "seg_2-x.0" => Ok(Citation { prefix: String::from("seg_2-x"), ordinal: 0 }),
        "a3" => Err(Uncited::Unshaped(String::from("a3"))),
        ".3" => Err(Uncited::Unshaped(String::from(".3"))),
        "a.3.4" => Err(Uncited::Unshaped(String::from("a.3.4"))),
        "a.b" => Err(Uncited::Unnumbered(String::from("a.b"))),
        "a.99999999999" => Err(Uncited::Unnumbered(String::from("a.99999999999"))),
    ]
);
