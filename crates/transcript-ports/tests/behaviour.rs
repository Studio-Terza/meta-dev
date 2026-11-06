use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::prop_assert_eq;
use port_core::{Rooting, Run};
use transcript_ports::{Authorship, ClaudeCodeDir, Cursor, Read, Record, Session, Transcripts};

const DIRECTORY: &str = "/projects/work";
const SESSION: &str = "/projects/work/s1.jsonl";

struct Lines;

impl Lines {
    fn entry(kind: &str, uuid: &str, origin: &str, content: &str) -> String {
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:0{uuid}Z\",\"origin\":{origin},\"message\":{{\"content\":{content}}}}}\n"
        )
    }

    fn typed(uuid: &str) -> String {
        Self::entry("user", uuid, "{\"kind\":\"human\"}", "\"typed words\"")
    }

    fn answer(uuid: &str) -> String {
        Self::entry(
            "assistant",
            uuid,
            "null",
            "[{\"type\":\"text\",\"text\":\"first\"},{\"type\":\"tool_use\"},{\"type\":\"text\",\"text\":\"second\"}]",
        )
    }

    fn tool_result(uuid: &str) -> String {
        Self::entry("user", uuid, "null", "[{\"type\":\"tool_result\",\"content\":\"output\"}]")
    }

    fn notice(uuid: &str) -> String {
        Self::entry("user", uuid, "{\"kind\":\"task-notification\"}", "\"finished\"")
    }

    fn skipped() -> String {
        "{\"type\":\"permission-mode\",\"sessionId\":\"s1\"}\n".to_owned()
    }
}

struct Transcript(String);

impl Transcript {
    fn holding(text: &str) -> Transcript {
        Transcript(text.to_owned())
    }

    fn directory(&self) -> ClaudeCodeDir<Memory> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY)).ok();
        memory.append(Path::new(SESSION), &Bytes::from(&self.0)).ok();
        ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY))
    }

    fn read(&self, cursor: Cursor) -> Option<Read> {
        let run = Run::root(Rooting::Shared).ok()?;
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port).ok()?;
        let read = sessions.first().map(|session| Transcripts::read_from(&port, session, cursor));
        read?.ok()
    }

    fn sessions(&self) -> Vec<Session> {
        Run::root(Rooting::Shared)
            .ok()
            .and_then(|run| Transcripts::sessions(&run.port(self.directory())).ok())
            .unwrap_or_default()
    }
}

fn authors(read: &Read) -> Vec<Authorship> {
    read.records.iter().map(|record| record.author).collect()
}

fn uuids(records: &[Record]) -> Vec<String> {
    records.iter().map(|record| record.uuid.to_string()).collect()
}

#[test]
fn each_line_is_classified_by_who_wrote_it() {
    let text =
        [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default());
    assert_eq!(
        read.as_ref().map(authors),
        Some(vec![Authorship::Owner, Authorship::Lead, Authorship::Bystander, Authorship::Bystander])
    );
}

#[test]
fn a_lead_record_keeps_only_its_text_blocks() {
    let read = Transcript::holding(&Lines::answer("2")).read(Cursor::default());
    let texts: Vec<String> =
        read.iter().flat_map(|read| read.records.iter()).map(|record| record.text.to_string()).collect();
    assert_eq!(texts, ["first\nsecond"]);
}

#[test]
fn a_partial_last_line_is_not_yielded() {
    let whole = Lines::typed("1");
    let partial = Lines::answer("2");
    let cut = partial.chars().take(partial.len() / 2).collect::<String>();
    let read = Transcript::holding(&format!("{whole}{cut}")).read(Cursor::default());
    assert_eq!(read.as_ref().map(|read| uuids(&read.records)), Some(vec!["1".to_owned()]));
}

#[test]
fn a_session_is_listed_by_its_file_stem() {
    let sessions = Transcript::holding(&Lines::typed("1")).sessions();
    assert_eq!(sessions.iter().map(|session| session.id().to_string()).collect::<Vec<_>>(), ["s1"]);
}

laws::law!(resuming_from_the_cursor_yields_every_record_exactly_once, 0usize..400, split, {
    let lines = [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4")];
    let text = lines.concat();
    let cut = split.min(text.len());
    let empty = || Read { records: Vec::new(), cursor: Cursor::default() };
    let first =
        Transcript::holding(&text.chars().take(cut).collect::<String>()).read(Cursor::default()).unwrap_or_else(empty);
    let second = Transcript::holding(&text).read(first.cursor).unwrap_or_else(empty);
    let together: Vec<Record> = first.records.into_iter().chain(second.records).collect();
    prop_assert_eq!(uuids(&together), ["1", "2", "3", "4"]);
});
