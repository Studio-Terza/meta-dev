#![deny(clippy::wildcard_enum_match_arm)]

use grammar_anchor::{Anchoring, Unanchored};

struct Inks;

#[derive(Debug, PartialEq, Eq)]
struct Anchored {
    message: String,
    ordinal: Option<u32>,
}

impl Inks {
    fn anchored(text: &&str) -> Result<Anchored, Unanchored> {
        text.parse::<Anchoring>()
            .map(|anchoring| Anchored { message: anchoring.message().to_owned(), ordinal: anchoring.ordinal() })
    }
}

laws::table!(
    an_anchor_reads_as_a_message_id_or_one_with_a_segment_index,
    Inks::anchored,
    [
        "85b287b3-4d5e" => Ok(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: None }),
        "85b287b3-4d5e#0" => Ok(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: Some(0) }),
        "o1#12" => Ok(Anchored { message: "o1".to_owned(), ordinal: Some(12) }),
        "o1#" => Err(Unanchored::Unindexed("o1#".to_owned())),
        "o1#01" => Err(Unanchored::Unindexed("o1#01".to_owned())),
        "o1#2#3" => Err(Unanchored::Unindexed("o1#2#3".to_owned())),
        "#2" => Err(Unanchored::Messageless("#2".to_owned())),
        "" => Err(Unanchored::Messageless(String::new())),
    ]
);
