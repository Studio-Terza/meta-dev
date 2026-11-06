use std::error::Error;
use std::path::{Path, PathBuf};

use design_model::{Entered, Entry, Facet, NodeId, NodeRelation, Project, Stage};
use design_store::{
    Linked, NewNode, ProjectRoot, Scope, SearchSegments, SessionCursors, SessionRecords, ShownMessage, ShownNode,
    StreamSchema,
};
use file_ports::{Bytes, Files, Memory};
use grammar::search::Terms;
use newtype::Counter;
use port_core::{Port, Run};
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
        Recorded::of(SESSION)
    }

    fn of(lines: &str) -> Result<Recorded, Box<dyn Error>> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY))?;
        memory.write(&Path::new(DIRECTORY).join("s.jsonl"), Bytes::from(lines))?;
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

struct Said;

impl Said {
    fn owner(uuid: &str, second: usize, text: &str) -> String {
        format!(
            "{{\"type\":\"user\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:0{second}Z\",\"origin\":{{\"kind\":\"human\"}},\"message\":{{\"content\":\"{text}\"}}}}\n"
        )
    }

    fn lead(uuid: &str, text: &str) -> String {
        format!(
            "{{\"type\":\"assistant\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:09Z\",\"origin\":null,\"message\":{{\"model\":\"claude\",\"content\":[{{\"type\":\"text\",\"text\":\"{text}\"}}]}}}}\n"
        )
    }

    fn found(lines: &[String], words: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        Said::scoped(lines, words, Scope::Owner)
    }

    fn scoped(lines: &[String], words: &[&str], scope: Scope) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(Recorded::of(&lines.concat())?.records)?;
        let terms = Terms::lexed(words.iter().copied()).ok_or("no terms")?;
        let hits = store.read(SearchSegments::matching(terms, scope))?;
        Ok(hits
            .iter()
            .map(|hit| format!("{} {} {} {}", hit.uuid(), hit.author(), hit.stamp(), Body::from(hit)))
            .collect())
    }
}

#[test]
fn a_search_returns_each_segment_holding_the_word_with_its_message() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "alpha here"), Said::owner("o2", 2, "beta there")];
    assert_eq!(Said::found(&lines, &["beta"])?, ["o2 owner 2026-01-01T00:00:02Z beta there"]);
    Ok(())
}

#[test]
fn several_terms_rank_a_segment_dense_in_the_rarer_term_first() -> Result<(), Box<dyn Error>> {
    let lines = [
        Said::owner("o1", 1, "alpha alpha alpha alpha beta"),
        Said::owner("o2", 2, "alpha beta beta beta beta"),
        Said::owner("o3", 3, "alpha"),
        Said::owner("o4", 4, "gamma"),
        Said::owner("o5", 5, "gamma"),
        Said::owner("o6", 6, "gamma"),
        Said::owner("o7", 7, "gamma"),
        Said::owner("o8", 8, "gamma"),
    ];
    let found = Said::found(&lines, &["alpha", "beta"])?;
    assert_eq!(
        found,
        [
            "o2 owner 2026-01-01T00:00:02Z alpha beta beta beta beta",
            "o1 owner 2026-01-01T00:00:01Z alpha alpha alpha alpha beta"
        ]
    );
    Ok(())
}

#[test]
fn several_terms_rank_a_segment_dense_in_both_before_one_diluted() -> Result<(), Box<dyn Error>> {
    let lines = [
        Said::owner("o1", 1, "delta epsilon zeta zeta zeta zeta zeta zeta zeta zeta"),
        Said::owner("o2", 2, "delta epsilon"),
        Said::owner("o3", 3, "gamma"),
        Said::owner("o4", 4, "gamma"),
        Said::owner("o5", 5, "gamma"),
        Said::owner("o6", 6, "gamma"),
        Said::owner("o7", 7, "gamma"),
        Said::owner("o8", 8, "gamma"),
    ];
    let found = Said::found(&lines, &["delta", "epsilon"])?;
    assert_eq!(found.first().map(String::as_str), Some("o2 owner 2026-01-01T00:00:02Z delta epsilon"));
    Ok(())
}

#[test]
fn the_lead_is_found_only_in_the_everyone_scope() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "omega"), Said::lead("l1", "omega")];
    assert_eq!(Said::scoped(&lines, &["omega"], Scope::Owner)?, ["o1 owner 2026-01-01T00:00:01Z omega"]);
    assert_eq!(Said::scoped(&lines, &["omega"], Scope::Everyone)?.len(), 2);
    Ok(())
}

struct Projects;

impl Projects {
    fn rooted(store: &Port<'_, Sqlite>, name: &str) -> Result<NodeId, Box<dyn Error>> {
        let project: Project = name.parse()?;
        store.write(NewNode::from(Entry::new(project.clone(), Facet::Project, Stage::Vague, Entered::Init.into())))?;
        Ok(store.read(ProjectRoot::from(project))?.ok_or("no root")?)
    }
}

#[test]
fn a_project_root_is_found_by_its_project_alone() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    Projects::rooted(&store, "q")?;
    let rooted = Projects::rooted(&store, "p")?;
    assert_eq!(rooted, NodeId::new(2));
    assert_eq!(store.read(ProjectRoot::from("r".parse::<Project>()?))?, None);
    Ok(())
}

#[test]
fn a_consumes_link_is_listed_from_its_source_only() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let eater = Projects::rooted(&store, "p")?;
    let consumed = Projects::rooted(&store, "q")?;
    store.write(Linked::new(eater, NodeRelation::Consumes, consumed))?;
    let shown = store.read(ShownNode::from(eater))?.ok_or("no consumer shown")?;
    let links: Vec<(NodeRelation, NodeId)> =
        shown.links().iter().map(|link| (link.relation(), link.target())).collect();
    assert_eq!(links, [(NodeRelation::Consumes, consumed)]);
    assert_eq!(shown.node().identity(), eater);
    assert_eq!(store.read(ShownNode::from(consumed))?.ok_or("no consumed shown")?.links(), []);
    Ok(())
}

#[test]
fn linking_the_same_nodes_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let eater = Projects::rooted(&store, "p")?;
    let consumed = Projects::rooted(&store, "q")?;
    let first = store.write(Linked::new(eater, NodeRelation::Consumes, consumed))?;
    let second = store.write(Linked::new(eater, NodeRelation::Consumes, consumed))?;
    assert!(first > Written::zero());
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn an_unknown_node_is_not_shown() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    Projects::rooted(&store, "p")?;
    assert!(store.read(ShownNode::from(NodeId::new(99)))?.is_none());
    Ok(())
}
