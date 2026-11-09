use std::error::Error;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::{prop_assert, prop_assert_eq};
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use port_core::Run;
use stream::{Anchor, Message, Relation, Shape, Stream};
use transcript_ports::{Authorship, ClaudeCodeDir, Cursor, Transcripts, Uuid};

const CITED: &str = "85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b";
const DIRECTORY: &str = "/projects/work";

struct Wire {
    kind: &'static str,
    origin: &'static str,
}

impl Wire {
    const SYSTEM: Wire = Wire { kind: "system", origin: "null" };
    const NOTICE: Wire = Wire { kind: "user", origin: "{\"kind\":\"task-notification\"}" };
    const UNHEARD: Wire = Wire { kind: "brand-new-kind", origin: "null" };

    fn of(author: Authorship) -> Wire {
        match author {
            Authorship::Owner => Wire { kind: "user", origin: "{\"kind\":\"human\"}" },
            Authorship::Lead => Wire { kind: "assistant", origin: "null" },
        }
    }

    fn content(author: Authorship, text: &str) -> String {
        match author {
            Authorship::Owner => format!("{text:?}"),
            Authorship::Lead => format!("[{{\"type\":\"text\",\"text\":{text:?}}}]"),
        }
    }
}

struct Exchange<'said> {
    led: &'said [Turn],
    replied: &'static str,
}

#[derive(Clone, Copy)]
enum Turn {
    Said(Authorship, &'static str),
    Acts,
    System,
    Notice,
    Unheard,
}

const OWNER: Turn = Turn::Said(Authorship::Owner, "words");
const LEAD: Turn = Turn::Said(Authorship::Lead, "words");
const ASKS: Turn = Turn::Said(Authorship::Lead, "1. a\n2. b");
const ANSWERS: Turn = Turn::Said(Authorship::Owner, "1. yes\n2. no");

impl Turn {
    const fn lead(text: &'static str) -> Turn {
        Turn::Said(Authorship::Lead, text)
    }
}

struct Say<'said> {
    uuid: &'said str,
    parent: Option<&'said str>,
    wire: Wire,
    content: String,
}

impl<'said> Say<'said> {
    fn new(uuid: &'said str, parent: Option<&'said str>, author: Authorship, text: &'said str) -> Say<'said> {
        Say { uuid, parent, wire: Wire::of(author), content: Wire::content(author, text) }
    }

    fn turn(uuid: &'said str, parent: Option<&'said str>, turn: Turn) -> Say<'said> {
        match turn {
            Turn::Said(author, text) => Say::new(uuid, parent, author, text),
            Turn::Acts => Say {
                uuid,
                parent,
                wire: Wire::of(Authorship::Lead),
                content: "[{\"type\":\"tool_use\",\"name\":\"Bash\"}]".to_owned(),
            },
            Turn::System => Say { uuid, parent, wire: Wire::SYSTEM, content: "\"\"".to_owned() },
            Turn::Notice => Say { uuid, parent, wire: Wire::NOTICE, content: "\"finished\"".to_owned() },
            Turn::Unheard => Say { uuid, parent, wire: Wire::UNHEARD, content: "\"\"".to_owned() },
        }
    }

    fn line(&self) -> String {
        let Wire { kind, origin } = self.wire;
        let parent = self.parent.map_or_else(|| "null".to_owned(), |parent| format!("{parent:?}"));
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":{:?},\"parentUuid\":{parent},\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"model\":\"claude\",\"content\":{}}}}}\n",
            self.uuid, self.content
        )
    }
}

struct Fixture;

impl Fixture {
    fn stream(said: &[Say<'_>]) -> Result<Stream, Box<dyn Error>> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY))?;
        memory
            .write(Path::new("/projects/work/s.jsonl"), Bytes::from(&said.iter().map(Say::line).collect::<String>()))?;
        let run = Run::root();
        let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY)));
        let sessions = Transcripts::sessions(&port)?;
        let reads = sessions
            .iter()
            .map(|session| Transcripts::read_from(&port, session, Cursor::default()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(reads.into_iter().flatten().collect())
    }

    fn related(relation: Relation, said: &[Say<'_>]) -> Vec<String> {
        Self::stream(said)
            .map(|stream| {
                stream.links().iter().filter(|link| link.relation() == relation).map(ToString::to_string).collect()
            })
            .unwrap_or_default()
    }

    fn first(text: &str) -> Result<Message, Box<dyn Error>> {
        let stream = Self::stream(&[Say::new("m", None, Authorship::Owner, text)])?;
        stream.as_ref().first().cloned().ok_or_else(|| "no message".into())
    }
}

struct Cases;

impl Cases {
    fn answers(exchange: &Exchange<'_>) -> Vec<String> {
        let turns: Vec<Turn> =
            exchange.led.iter().copied().chain([Turn::Said(Authorship::Owner, exchange.replied)]).collect();
        Fixture::related(Relation::Answers, &Self::chained(&Self::ids(&turns), &turns))
    }

    fn cites(text: &&str) -> Vec<String> {
        Fixture::related(
            Relation::Cites,
            &[
                Say::new(CITED, None, Authorship::Lead, "earlier"),
                Say::new("owner", Some(CITED), Authorship::Owner, text),
            ],
        )
    }

    fn shapes(text: &&str) -> Vec<String> {
        Fixture::first(text)
            .map(|message| message.segments().iter().map(|segment| Shape::from(segment.form()).to_string()).collect())
            .unwrap_or_default()
    }

    fn bodies(text: &&str) -> Vec<String> {
        Fixture::first(text)
            .map(|message| {
                message
                    .segments()
                    .iter()
                    .map(|segment| segment.text(&message).map_or_else(|nul| nul.to_string(), |body| body.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn resends(texts: &[&str]) -> Vec<String> {
        let ids: Vec<String> = (0..texts.len()).map(|position| format!("m{position}")).collect();
        let said: Vec<Say<'_>> =
            ids.iter().zip(texts).map(|(id, text)| Say::new(id, None, Authorship::Owner, text)).collect();
        Fixture::related(Relation::Resends, &said)
    }

    fn resends_across_authors(authors: &[Authorship]) -> Vec<String> {
        let ids: Vec<String> = (0..authors.len()).map(|position| format!("m{position}")).collect();
        let said: Vec<Say<'_>> =
            ids.iter().zip(authors).map(|(id, author)| Say::new(id, None, *author, "same")).collect();
        Fixture::related(Relation::Resends, &said)
    }

    fn said(chain: &[Turn]) -> Vec<Say<'static>> {
        chain.iter().map(|turn| Say::turn("m", None, *turn)).collect()
    }

    fn unrecognised(chain: &[Turn]) -> Vec<String> {
        Fixture::stream(&Self::said(chain))
            .map(|stream| stream.unrecognised().iter().map(|unrecognised| unrecognised.kind().to_string()).collect())
            .unwrap_or_default()
    }

    fn ids(chain: &[Turn]) -> Vec<String> {
        (0..chain.len()).map(|position| format!("m{position}")).collect()
    }

    fn chained<'said>(ids: &'said [String], chain: &[Turn]) -> Vec<Say<'said>> {
        ids.iter()
            .zip(chain)
            .enumerate()
            .map(|(position, (id, turn))| {
                Say::turn(id, position.checked_sub(1).and_then(|before| ids.get(before)).map(String::as_str), *turn)
            })
            .collect()
    }

    fn replies(chain: &[Turn]) -> Vec<String> {
        let ids = Self::ids(chain);
        let said = Self::chained(&ids, chain);
        [Relation::RepliesTo, Relation::Answers]
            .into_iter()
            .flat_map(|relation| Fixture::related(relation, &said))
            .collect()
    }
}

struct Touching;

impl Touching {
    fn linked(touched: &&str) -> Vec<String> {
        Fixture::stream(&[
            Say::new("o1", None, Authorship::Owner, "1. ask"),
            Say::new("l1", Some("o1"), Authorship::Lead, "1. reply"),
            Say::new("o2", Some("l1"), Authorship::Owner, "1. yes"),
        ])
        .and_then(|stream| {
            let uuid = Uuid::try_new((*touched).to_owned())?;
            Ok(stream
                .links()
                .iter()
                .filter(|link| link.anchors().into_iter().any(|anchor| *anchor.message() == uuid))
                .map(ToString::to_string)
                .collect())
        })
        .unwrap_or_default()
    }
}

laws::table!(
    a_link_touches_the_messages_at_both_its_ends_and_their_segments,
    Touching::linked,
    [
        "o1" => ["l1 replies-to o1"],
        "o2" => ["o2 replies-to l1", "o2#0 answers l1#0"],
        "l1" => ["l1 replies-to o1", "o2 replies-to l1", "o2#0 answers l1#0"],
        "nobody" => Vec::<String>::new(),
    ]
);

laws::law!(segments_tile_the_ink_of_the_message_without_loss_or_stray_whitespace, "[ -~\n\t•]{0,80}", text, {
    let message = Fixture::first(&text).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let bodies: Vec<String> = message
        .segments()
        .iter()
        .map(|segment| segment.text(&message).map_or_else(|nul| nul.to_string(), |body| body.to_string()))
        .collect();
    let inked = |text: &str| text.chars().filter(|letter| !letter.is_whitespace()).collect::<String>();
    prop_assert_eq!(inked(&bodies.concat()), inked(&text));
    let ends = |body: &String| [body.chars().next(), body.chars().last()].into_iter().flatten().collect::<Vec<char>>();
    prop_assert!(bodies.iter().flat_map(ends).all(|end| !end.is_whitespace()));
});

laws::table!(
    a_segment_is_lexed_without_stray_leading_or_trailing_whitespace,
    Cases::bodies,
    [
        "Hello there.  How are you? \n1. one \n  - two\t\n> three \n" => ["Hello there.", "How are you?", "1. one", "- two", "> three"],
        "  padded  " => ["padded"],
        "1. yes\n2. no" => ["1. yes", "2. no"],
    ]
);

laws::table!(
    numbered_answers_align_with_the_last_numbered_list_of_the_lead_turn,
    Cases::answers,
    [
        Exchange { led: &[ASKS], replied: "1. yes\n2. no" } => ["m1#0 answers m0#0", "m1#1 answers m0#1"],
        Exchange { led: &[ASKS], replied: "2. no" } => ["m1#0 answers m0#1"],
        Exchange { led: &[Turn::lead("1. a")], replied: "3. none" } => Vec::<String>::new(),
        Exchange { led: &[Turn::lead("a question?")], replied: "1. yes" } => Vec::<String>::new(),
        Exchange { led: &[Turn::lead("Findings:\n1. a\n2. b\n\nQuestions:\n1. c\n2. d")], replied: "1. yes" } => ["m1#0 answers m0#4"],
        Exchange { led: &[Turn::lead("Intro\n  2. nested\n1. a\n2. b")], replied: "2. no" } => ["m1#0 answers m0#3"],
        Exchange { led: &[Turn::lead("1. a\n  1. sub\n2. b")], replied: "1. yes\n2. no" } => ["m1#0 answers m0#0", "m1#1 answers m0#2"],
        Exchange { led: &[ASKS, Turn::Acts, Turn::lead("Which do you want?")], replied: "1. yes\n2. no" } => ["m3#0 answers m0#0", "m3#1 answers m0#1"],
        Exchange { led: &[ASKS, LEAD, Turn::Notice, LEAD, LEAD], replied: "2. no" } => ["m5#0 answers m0#1"],
        Exchange { led: &[LEAD, ASKS, LEAD], replied: "1. yes" } => ["m3#0 answers m1#0"],
        Exchange { led: &[Turn::lead("1. x\n2. y"), Turn::Acts, ASKS, LEAD], replied: "1. yes" } => ["m4#0 answers m2#0"],
        Exchange { led: &[Turn::lead("**1. a**\nwhy\n**2. b**\nhow")], replied: "2. no" } => ["m1#0 answers m0#2"],
        Exchange { led: &[ASKS, OWNER, LEAD], replied: "1. yes" } => Vec::<String>::new(),
    ]
);

laws::table!(
    a_known_message_uuid_in_a_segment_is_cited,
    Cases::cites,
    [
        "see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b here" => ["owner#0 cites 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b"],
        "see 11111111-2222-3333-4444-555555555555 here" => Vec::<String>::new(),
        "nothing to cite" => Vec::<String>::new(),
    ]
);

laws::table!(
    identical_text_by_the_same_author_resends_the_first,
    Cases::resends,
    [
        ["same", "same"][..] => ["m1 resends m0"],
        ["a", "b", "a", "a"][..] => ["m2 resends m0", "m3 resends m0"],
        ["", ""][..] => Vec::<String>::new(),
    ]
);

laws::table!(
    the_same_text_by_another_author_is_not_a_resend,
    Cases::resends_across_authors,
    [
        [Authorship::Owner, Authorship::Lead][..] => Vec::<String>::new(),
        [Authorship::Owner, Authorship::Owner][..] => ["m1 resends m0"],
    ]
);

laws::table!(
    a_reply_skips_the_records_between,
    Cases::replies,
    [
        [OWNER, LEAD, Turn::Notice, LEAD, OWNER][..]
            => ["m1 replies-to m0", "m3 replies-to m1", "m4 replies-to m3"],
        [ASKS, Turn::System, ANSWERS][..]
            => ["m2 replies-to m0", "m2#0 answers m0#0", "m2#1 answers m0#1"],
        [OWNER, Turn::Acts, LEAD][..] => ["m2 replies-to m0"],
        [OWNER, ASKS, Turn::Unheard, ANSWERS][..]
            => ["m1 replies-to m0", "m3 replies-to m1", "m3#0 answers m1#0", "m3#1 answers m1#1"],
        [LEAD][..] => Vec::<String>::new(),
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

laws::table!(
    a_stream_counts_each_unrecognised_record,
    Cases::unrecognised,
    [
        [OWNER, Turn::Unheard, Turn::Unheard][..] => ["brand-new-kind", "brand-new-kind"],
        [OWNER, LEAD][..] => Vec::<String>::new(),
    ]
);

laws::roundtrip!(
    every_anchor_is_displayed_and_parsed_back,
    "[0-9a-f-]{1,36}(#(0|[1-9][0-9]{0,5}))?".prop_filter_map("an anchor", |text| text.parse::<Anchor>().ok()),
    |anchor: &Anchor| anchor.to_string(),
    |text: &String| text.parse::<Anchor>().ok(),
    Some
);
