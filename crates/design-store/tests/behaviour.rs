use std::error::Error;
use std::path::{Path, PathBuf};

use design_store::{SessionCursors, SessionRecords, ShownMessage, StreamSchema};
use file_ports::{Bytes, Files, Memory};
use newtype::Counter;
use port_core::Run;
use store_ports::{Sqlite, Store, Written};
use stream::Stream;
use transcript_ports::{Body, ClaudeCodeDir, Cursor, SessionId, Transcripts, Uuid};

const DIRECTORY: &str = "/projects/work";

const SESSION: &str = concat!(
    "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"1. go\\n2. now\"}}\n",
    "{\"type\":\"assistant\",\"uuid\":\"l1\",\"parentUuid\":\"o1\",\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:01Z\",\"origin\":null,\"message\":{\"model\":\"claude\",\"content\":[{\"type\":\"text\",\"text\":\"done\"}]}}\n"
);

struct Recorded {
    records: SessionRecords,
    cursor: Cursor,
}

impl Recorded {
    fn session() -> Result<Recorded, Box<dyn Error>> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY))?;
        memory.write(&Path::new(DIRECTORY).join("s.jsonl"), Bytes::from(SESSION))?;
        let run = Run::root();
        let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY)));
        let sessions = port.sessions()?;
        let session = sessions.first().ok_or("no session")?;
        let read = port.read_from(session, Cursor::default())?;
        let cursor = read.cursor();
        let stream: Stream = read.into_iter().collect();
        Ok(Recorded { records: SessionRecords::read(session.stem(), cursor, &stream), cursor })
    }
}

#[test]
fn a_written_session_reads_back_with_its_segments_links_and_cursor() -> Result<(), Box<dyn Error>> {
    let Recorded { records, cursor } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let shown = store.read(ShownMessage::named(Uuid::new("o1".to_owned())))?.ok_or("o1 was not stored")?;
    let message = shown.message();
    assert_eq!(
        [
            message.uuid().to_string(),
            message.author().to_string(),
            message.session().to_string(),
            message.stamp().to_string()
        ],
        ["o1", "owner", "s", "2026-01-01T00:00:00Z"]
    );
    let segments: Vec<String> =
        shown.segments().iter().map(|segment| format!("{} {}", segment.ordinal(), segment.shape())).collect();
    assert_eq!(segments, ["0 item", "1 item"]);
    let first = shown.segments().first().map(Body::from).ok_or("no segment")?;
    assert_eq!(first.to_string(), "1. go");
    let links: Vec<String> = shown
        .links()
        .iter()
        .map(|link| {
            let [source, target] = link.anchors();
            format!("{source} {} {target}", link.relation())
        })
        .collect();
    assert_eq!(links, ["l1 replies-to o1"]);
    let cursors = store.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), cursor);
    Ok(())
}

#[test]
fn writing_the_same_session_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let first = store.write(Recorded::session()?.records)?;
    let second = store.write(Recorded::session()?.records)?;
    assert!(first.count() > 0);
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn an_unknown_message_is_not_shown() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(Recorded::session()?.records)?;
    assert!(store.read(ShownMessage::named(Uuid::new("nobody".to_owned())))?.is_none());
    Ok(())
}
