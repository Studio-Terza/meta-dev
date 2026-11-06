use std::error::Error;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::test_runner::TestCaseError;
use port_core::{Calls, Run};
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, ClaudeCodeDir, Cursor, Gather, Instant, Read, Record, Session, Skipped, Transcripts,
};

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

    fn system(uuid: &str, parent: &str) -> String {
        format!(
            "{{\"type\":\"system\",\"subtype\":\"turn_duration\",\"uuid\":\"{uuid}\",\"parentUuid\":\"{parent}\",\"sessionId\":\"s1\"}}\n"
        )
    }

    fn skipped() -> String {
        "{\"type\":\"permission-mode\",\"sessionId\":\"s1\"}\n".to_owned()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Counted {
    sessions: Calls,
    reads: Calls,
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

    fn read(&self, cursor: Cursor) -> Result<Read, Box<dyn Error>> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port)?;
        let session = sessions.first().ok_or("no session")?;
        Ok(Transcripts::read_from(&port, session, cursor)?)
    }

    fn sessions(&self) -> Vec<Session> {
        Transcripts::sessions(&Run::root().port(self.directory())).unwrap_or_default()
    }

    fn calls(&self) -> Option<Counted> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port).ok()?;
        Transcripts::read_all(&port, &sessions).ok()?;
        drop(port);
        let measured = run.finish();
        Some(Counted {
            sessions: measured.snapshot().calls::<Sessions>(),
            reads: measured.snapshot().calls::<ReadFrom>(),
        })
    }
}

#[derive(Default)]
struct Gathered {
    records: Vec<Record>,
    skipped: Vec<Skipped>,
}

impl Gather for Gathered {
    fn said(mut self, record: Record) -> Gathered {
        self.records.push(record);
        self
    }

    fn skipped(mut self, skipped: Skipped) -> Gathered {
        self.skipped.push(skipped);
        self
    }
}

impl From<Read> for Gathered {
    fn from(read: Read) -> Gathered {
        read.into_iter().fold(Gathered::default(), |gathered, line| line.gathered(gathered))
    }
}

struct Views;

impl Views {
    fn records(read: Read) -> Vec<Record> {
        Gathered::from(read).records
    }

    fn edges(read: Read) -> Vec<String> {
        Gathered::from(read)
            .skipped
            .into_iter()
            .filter_map(|skipped| skipped.parent().map(|parent| format!("{} after {parent}", skipped.uuid())))
            .collect()
    }

    fn authors(read: Read) -> Vec<Authorship> {
        Self::records(read).into_iter().map(|record| record.author()).collect()
    }

    fn uuids(records: &[Record]) -> Vec<String> {
        records.iter().map(|record| record.uuid().to_string()).collect()
    }

    fn texts(read: Read) -> Vec<String> {
        Self::records(read).into_iter().map(|record| record.text().to_string()).collect()
    }
}

#[test]
fn each_line_is_classified_by_who_wrote_it() -> Result<(), Box<dyn Error>> {
    let text =
        [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(
        Views::authors(read),
        [Authorship::Owner, Authorship::Lead, Authorship::Bystander, Authorship::Bystander]
    );
    Ok(())
}

#[test]
fn a_line_that_is_not_a_message_yields_its_parent_edge() -> Result<(), Box<dyn Error>> {
    let text = [Lines::answer("2"), Lines::system("5", "2"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(Views::edges(read), ["5 after 2"]);
    Ok(())
}

#[test]
fn a_lead_record_keeps_only_its_text_blocks() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::answer("2")).read(Cursor::default())?;
    assert_eq!(Views::texts(read), ["first second"]);
    Ok(())
}

#[test]
fn a_record_carries_the_timestamp_of_its_line() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::typed("1")).read(Cursor::default())?;
    let stamps: Vec<Instant> = Views::records(read).into_iter().map(|record| record.stamp().clone()).collect();
    assert_eq!(stamps, [Instant::new("2026-01-01T00:00:01Z".to_owned())]);
    Ok(())
}

#[test]
fn a_partial_last_line_is_not_yielded() -> Result<(), Box<dyn Error>> {
    let whole = Lines::typed("1");
    let partial = Lines::answer("2");
    let cut = partial.chars().take(partial.len() / 2).collect::<String>();
    let read = Transcript::holding(&format!("{whole}{cut}")).read(Cursor::default())?;
    let records = Views::records(read);
    assert_eq!(Views::uuids(&records), ["1"]);
    Ok(())
}

#[test]
fn a_record_uuid_reads_as_its_text() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::typed("abc")).read(Cursor::default())?;
    let texts: Vec<String> = Views::records(read).into_iter().map(|record| record.uuid().to_string()).collect();
    assert_eq!(texts, ["abc"]);
    Ok(())
}

#[test]
fn a_session_is_listed_by_its_file_stem() {
    let sessions = Transcript::holding(&Lines::typed("1")).sessions();
    assert_eq!(sessions.iter().map(|session| session.stem().to_string()).collect::<Vec<_>>(), ["s1"]);
}

#[test]
fn reading_all_sessions_counts_one_listing_and_one_read_per_session() {
    let counted = Transcript::holding(&Lines::typed("1")).calls();
    assert_eq!(counted, Some(Counted { sessions: Calls::new(1), reads: Calls::new(1) }));
}

laws::law!(resuming_from_the_cursor_yields_every_record_exactly_once, 0usize..400, split, {
    let lines = [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4")];
    let text = lines.concat();
    let cut = split.min(text.len());
    let first = Transcript::holding(&text.chars().take(cut).collect::<String>())
        .read(Cursor::default())
        .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let second =
        Transcript::holding(&text).read(first.cursor()).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let together: Vec<Record> = Views::records(first).into_iter().chain(Views::records(second)).collect();
    prop_assert_eq!(Views::uuids(&together), ["1", "2", "3", "4"]);
});
