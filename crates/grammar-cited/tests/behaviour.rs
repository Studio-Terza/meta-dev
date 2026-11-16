#![deny(clippy::wildcard_enum_match_arm)]

use grammar_cited::{Cited, Uncited};

struct Citations;

impl Citations {
    fn read(text: &&str) -> Result<(String, u32), Uncited> {
        text.parse::<Cited>().map(|cited| (cited.prefix().to_owned(), cited.ordinal()))
    }
}

laws::table!(
    a_citation_reads_as_a_prefix_and_an_ordinal,
    Citations::read,
    [
        "a.3" => Ok((String::from("a"), 3)),
        "seg_2-x.0" => Ok((String::from("seg_2-x"), 0)),
        "a3" => Err(Uncited::Unshaped(String::from("a3"))),
        ".3" => Err(Uncited::Unshaped(String::from(".3"))),
        "a.3.4" => Err(Uncited::Unshaped(String::from("a.3.4"))),
        "a.b" => Err(Uncited::Unnumbered(String::from("a.b"))),
        "a.99999999999" => Err(Uncited::Unnumbered(String::from("a.99999999999"))),
    ]
);
