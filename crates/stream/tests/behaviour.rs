use std::error::Error;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::test_runner::TestCaseError;
use port_core::{Rooting, Run};
use stream::{Message, Relation, Shape, Stream};
use transcript_ports::{Authorship, ClaudeCodeDir, Transcripts};

const CITED: &str = "85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b";
const DIRECTORY: &str = "/projects/work";

struct Wire {
    kind: &'static str,
    origin: &'static str,
}

impl Wire {
    fn of(author: Authorship) -> Wire {
        match author {
            Authorship::Owner => Wire { kind: "user", origin: "{\"kind\":\"human\"}" },
            Authorship::Lead => Wire { kind: "assistant", origin: "null" },
            Authorship::Bystander => Wire { kind: "user", origin: "null" },
        }
    }
}

struct Exchange<'said> {
    asked: &'said str,
    replied: &'said str,
}

struct Say<'said> {
    uuid: &'said str,
    parent: Option<&'said str>,
    author: Authorship,
    text: &'said str,
}

impl<'said> Say<'said> {
    fn new(uuid: &'said str, parent: Option<&'said str>, author: Authorship, text: &'said str) -> Say<'said> {
        Say { uuid, parent, author, text }
    }

    fn line(&self) -> String {
        let Wire { kind, origin } = Wire::of(self.author);
        let parent = self.parent.map_or_else(|| "null".to_owned(), |parent| format!("{parent:?}"));
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":{:?},\"parentUuid\":{parent},\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"content\":{:?}}}}}\n",
            self.uuid, self.text
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
        let run = Run::root(Rooting::Shared)?;
        let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY)));
        let sessions = Transcripts::sessions(&port)?;
        let reads = Transcripts::read_all(&port, &sessions)?;
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
        Fixture::related(
            Relation::Answers,
            &[
                Say::new("lead", None, Authorship::Lead, exchange.asked),
                Say::new("owner", Some("lead"), Authorship::Owner, exchange.replied),
            ],
        )
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

    fn replies(chain: &[Authorship]) -> Vec<String> {
        let ids: Vec<String> = (0..chain.len()).map(|position| format!("m{position}")).collect();
        let said: Vec<Say<'_>> = ids
            .iter()
            .zip(chain)
            .enumerate()
            .map(|(position, (id, author))| {
                Say::new(
                    id,
                    position.checked_sub(1).and_then(|before| ids.get(before)).map(String::as_str),
                    *author,
                    "words",
                )
            })
            .collect();
        Fixture::related(Relation::RepliesTo, &said)
    }
}

laws::law!(segments_tile_the_message_without_loss, "[ -~\n•]{0,80}", text, {
    let message = Fixture::first(&text).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let rebuilt: String = message.segments().iter().map(|segment| segment.text(&message).to_string()).collect();
    prop_assert_eq!(rebuilt, text);
});

laws::table!(
    numbered_answers_align_with_the_numbered_items_they_reply_to,
    Cases::answers,
    [
        Exchange { asked: "1. a\n2. b", replied: "1. yes\n2. no" } => ["owner#0 answers lead#0", "owner#1 answers lead#1"],
        Exchange { asked: "1. a\n2. b", replied: "2. no" } => ["owner#0 answers lead#1"],
        Exchange { asked: "1. a", replied: "3. none" } => Vec::<String>::new(),
        Exchange { asked: "a question?", replied: "1. yes" } => Vec::<String>::new(),
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
    a_reply_skips_the_bystander_records_between,
    Cases::replies,
    [
        [Authorship::Owner, Authorship::Lead, Authorship::Bystander, Authorship::Lead, Authorship::Owner][..]
            => ["m1 replies-to m0", "m3 replies-to m1", "m4 replies-to m3"],
        [Authorship::Lead][..] => Vec::<String>::new(),
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
