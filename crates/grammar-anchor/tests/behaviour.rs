#![deny(clippy::wildcard_enum_match_arm)]

use grammar_anchor::Anchoring;

struct Inks;

#[derive(Debug, PartialEq, Eq)]
struct Anchored {
    message: String,
    ordinal: Option<usize>,
}

impl Inks {
    fn anchored(text: &&str) -> Option<Anchored> {
        Anchoring::read(text)
            .map(|anchoring| Anchored { message: anchoring.message().to_owned(), ordinal: anchoring.ordinal() })
    }
}

laws::table!(
    an_anchor_reads_as_a_message_id_or_one_with_a_segment_index,
    Inks::anchored,
    [
        "85b287b3-4d5e" => Some(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: None }),
        "85b287b3-4d5e#0" => Some(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: Some(0) }),
        "o1#12" => Some(Anchored { message: "o1".to_owned(), ordinal: Some(12) }),
        "o1#" => None,
        "o1#01" => None,
        "o1#2#3" => None,
        "#2" => None,
        "" => None,
    ]
);
