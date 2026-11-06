use laws::proptest::prelude::prop_assert_eq;
use stream::{Anchor, MechanicalLink, Message, Relation, Shape, Stream};
use transcript_ports::{Authorship, Body, Instant, Record, Uuid};

const CITED: &str = "85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b";

struct Fixture;

impl Fixture {
    fn record(uuid: &str, parent: Option<&str>, author: Authorship, text: &str) -> Record {
        Record {
            uuid: Uuid::new(uuid.to_owned()),
            parent: parent.map(|parent| Uuid::new(parent.to_owned())),
            session: Uuid::new("session".to_owned()),
            at: Instant::new("2026-01-01T00:00:00Z".to_owned()),
            author,
            text: Body::new(text.to_owned()),
        }
    }

    fn stream(records: Vec<Record>) -> Stream {
        records.into_iter().collect()
    }

    fn anchor(anchor: &Anchor) -> String {
        match anchor {
            Anchor::Message(uuid) => uuid.to_string(),
            Anchor::Segment(id) => format!("{}#{}", id.message(), id.ordinal()),
        }
    }

    fn of(relation: Relation, stream: &Stream) -> Vec<String> {
        stream
            .links()
            .iter()
            .filter(|link| link.relation == relation)
            .map(|link: &MechanicalLink| format!("{}>{}", Self::anchor(&link.from), Self::anchor(&link.to)))
            .collect()
    }
}

struct Cases;

impl Cases {
    fn answers(texts: &(&str, &str)) -> Vec<String> {
        let stream = Fixture::stream(vec![
            Fixture::record("lead", None, Authorship::Lead, texts.0),
            Fixture::record("owner", Some("lead"), Authorship::Owner, texts.1),
        ]);
        Fixture::of(Relation::Answers, &stream)
    }

    fn cites(text: &&str) -> Vec<String> {
        let stream = Fixture::stream(vec![
            Fixture::record(CITED, None, Authorship::Lead, "earlier"),
            Fixture::record("owner", Some(CITED), Authorship::Owner, text),
        ]);
        Fixture::of(Relation::Cites, &stream)
    }

    fn shapes(text: &&str) -> Vec<String> {
        Message::from(Fixture::record("m", None, Authorship::Owner, text))
            .segments()
            .iter()
            .map(|segment| Shape::from(segment.form).to_string())
            .collect()
    }

    fn resends(sent: &[(Authorship, &str)]) -> Vec<String> {
        let records = sent
            .iter()
            .enumerate()
            .map(|(position, (author, text))| Fixture::record(&format!("m{position}"), None, *author, text))
            .collect();
        Fixture::of(Relation::Resends, &Fixture::stream(records))
    }

    fn replies(chain: &[(&str, &str, Authorship)]) -> Vec<String> {
        let records = chain
            .iter()
            .map(|(uuid, parent, author)| {
                Fixture::record(uuid, Some(parent).filter(|name| !name.is_empty()).copied(), *author, "words")
            })
            .collect();
        Fixture::of(Relation::RepliesTo, &Fixture::stream(records))
    }
}

laws::law!(segments_tile_the_message_without_loss, "[ -~\n•]{0,80}", text, {
    let message = Message::from(Fixture::record("m", None, Authorship::Owner, &text));
    let rebuilt: String = message.segments().iter().map(|segment| segment.text(&message).to_string()).collect();
    prop_assert_eq!(rebuilt, text);
});

laws::table!(
    numbered_answers_align_with_the_numbered_items_they_reply_to,
    Cases::answers,
    [
        ("1. a\n2. b", "1. yes\n2. no") => ["owner#0>lead#0", "owner#1>lead#1"],
        ("1. a\n2. b", "2. no") => ["owner#0>lead#1"],
        ("1. a", "3. none") => Vec::<String>::new(),
        ("a question?", "1. yes") => Vec::<String>::new(),
    ]
);

laws::table!(
    a_known_message_uuid_in_a_segment_is_cited,
    Cases::cites,
    [
        "see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b here" => ["owner#0>85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b"],
        "see 11111111-2222-3333-4444-555555555555 here" => Vec::<String>::new(),
        "nothing to cite" => Vec::<String>::new(),
    ]
);

laws::table!(
    identical_text_by_the_same_author_resends_the_first,
    Cases::resends,
    [
        [(Authorship::Owner, "same"), (Authorship::Owner, "same")][..] => ["m1>m0"],
        [(Authorship::Owner, "same"), (Authorship::Lead, "same")][..] => Vec::<String>::new(),
        [(Authorship::Owner, "a"), (Authorship::Owner, "b"), (Authorship::Owner, "a"), (Authorship::Owner, "a")][..] => ["m2>m0", "m3>m0"],
        [(Authorship::Owner, "  "), (Authorship::Owner, "  ")][..] => Vec::<String>::new(),
    ]
);

laws::table!(
    a_reply_skips_the_bystander_records_between,
    Cases::replies,
    [
        [("o1", "", Authorship::Owner), ("l1", "o1", Authorship::Lead), ("t1", "l1", Authorship::Bystander), ("l2", "t1", Authorship::Lead), ("o2", "l2", Authorship::Owner)][..]
            => ["l1>o1", "l2>l1", "o2>l2"],
        [("l1", "missing", Authorship::Lead)][..] => Vec::<String>::new(),
    ]
);

laws::table!(
    a_message_splits_into_sentences_items_bullets_and_quotes,
    Cases::shapes,
    [
        "Hello there. How are you?\n1. one\n- two\n> three" => ["sentence", "sentence", "item", "bullet", "quote"],
        "1. first. still first.\n2. second" => ["item", "item"],
        "plain words" => ["sentence"],
        "\n\ntrailing\n\n" => ["sentence"],
        "" => Vec::<String>::new(),
    ]
);
