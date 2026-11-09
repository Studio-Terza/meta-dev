#![deny(clippy::wildcard_enum_match_arm)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use design_model::{
    AnyNode, Casting, Entered, Entry, Facet, JobId, Kin, Misstaged, Node, NodeEdge, NodeId, NodeRelation, Ordinal,
    Prefix, Project, Provenance, Role, RunId, Segment, SegmentRelation, Stage, Supersession, Title, Transition, Vague,
};
use design_store::{
    Account, AppliedFile, Cast, ContextNodes, Covered, Cycle, Directed, Directions, Excerpt, FailuresReported,
    FlailedSegments, Halted, HeldBy, Holding, JobFlailed, JobHalted, JobOpened, JobRepeated, JobRetried, Linked,
    LiveRuns, NextJob, NextNode, NextRun, OpenJobs, Placed, Planting, Prefixed, ProjectRoot, Ran, Refusal,
    RelationName, Remit, Resolved, Resolving, Restaged, Retold, RunEnded, RunSpawned, RunState, RunsOf, Scope,
    SearchSegments, Seen, SessionCursors, SessionRecords, ShapeName, ShownMessage, ShownNode, Spawned, StoredRun,
    StreamSchema, UnplacedSegments, UnreportedFailures, Unseen, Unstructured,
};
use dispatch_ports::{Deadline, Finished, Pid, Reply, Started, Tokens, Usage, Wall};
use file_ports::{Bytes, Files, Memory};
use grammar_message::Lexemes;
use grammar_search::Terms;
use laws::proptest::collection::vec;
use laws::proptest::option;
use laws::proptest::prelude::{Just, any, prop_assert, prop_assert_eq, prop_oneof};
use laws::proptest::sample::{Index, select};
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use newtype::{Checked, Counter, Display, Newtype, Wrapping};
use port_core::{Op, Port, Run};
use sqlite_ports::SqliteOpener;
use store_ports::{Batch, Columns, Migrator, Query, Schema, Statements, Store, StoreFailure, Version, Written};
use stream::{Message, Stream};
use strum::VariantArray;
use transcript_ports::{Authorship, Body, ClaudeCodeDir, Cursor, Instant, Nul, SessionId, Skipped, Transcripts, Uuid};

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
    store.write(store.read(Holding::from(records))?)?;
    let shown = store.read(ShownMessage::named(Uuid::try_new("o1".to_owned())?))?.ok_or("o1 was not stored")?;
    let message = shown.message();
    assert_eq!(
        [
            AsRef::<Uuid>::as_ref(message).to_string(),
            AsRef::<Authorship>::as_ref(message).to_string(),
            AsRef::<SessionId>::as_ref(message).to_string(),
            AsRef::<Instant>::as_ref(message).to_string()
        ],
        ["o1", "owner", "s", "2026-01-01T00:00:00Z"]
    );
    let segments: Vec<String> = shown
        .segments()
        .iter()
        .map(|segment| format!("{} {}", segment.ordinal(), AsRef::<ShapeName>::as_ref(segment)))
        .collect();
    assert_eq!(segments, ["0 item", "1 item"]);
    let first = shown.segments().first().map(Body::from).ok_or("no segment")?;
    assert_eq!(first.to_string(), "1. go");
    let links: Vec<String> = shown
        .links()
        .iter()
        .map(|link| {
            let [source, target] = link.anchors();
            format!("{source} {} {target}", AsRef::<RelationName>::as_ref(link))
        })
        .collect();
    assert_eq!(links, ["l1 replies-to o1"]);
    let cursors = store.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::try_new("s".to_owned())?), cursor);
    Ok(())
}

#[test]
fn writing_the_same_session_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let first = store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    let second = store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    assert!(first.count() > 0);
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn an_unknown_message_is_not_shown() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    assert!(store.read(ShownMessage::named(Uuid::try_new("nobody".to_owned())?))?.is_none());
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
        store.write(store.read(Holding::from(Recorded::of(&lines.concat())?.records))?)?;
        let terms = Terms::lexed(words.iter().copied()).ok_or("no terms")?;
        let hits = store.read(SearchSegments::matching(terms, scope))?;
        Ok(hits
            .iter()
            .map(|hit| {
                let segment = Segment::from(hit);
                let uuid: &Uuid = segment.as_ref();
                let author: &Authorship = hit.as_ref();
                let stamp: &Instant = hit.as_ref();
                format!("{uuid} {author} {stamp} {}", Body::from(hit))
            })
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

#[test]
fn a_stem_finds_its_inflected_forms() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "banning dyn seems desirable"), Said::owner("o2", 2, "we decided")];
    assert_eq!(Said::found(&lines, &["ban", "dyn"])?, ["o1 owner 2026-01-01T00:00:01Z banning dyn seems desirable"]);
    assert_eq!(Said::found(&lines, &["decide"])?, ["o2 owner 2026-01-01T00:00:02Z we decided"]);
    Ok(())
}

#[test]
fn an_unrelated_word_finds_nothing() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "banning dyn seems desirable"), Said::owner("o2", 2, "we decided")];
    assert_eq!(Said::found(&lines, &["kettle"])?, Vec::<String>::new());
    Ok(())
}

static SEEDS: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!("tests/seeded"));

static SEEDED: LazyLock<Migrator> =
    LazyLock::new(|| Migrator::from(StreamSchema::migrator().iter().chain(SEEDS.iter()).copied().collect::<Vec<_>>()));

#[derive(Clone, Copy, Debug)]
struct Seeded;

#[derive(Op)]
enum Seeding {
    Untrimmed,
    Streamed(<StreamSchema as Schema>::Rewrite),
}

impl Seeding {
    const UNTRIMMED: Version = Version::new(20_261_106_000_007);
}

impl Schema for Seeded {
    type Rewrite = Seeding;

    fn migrator() -> &'static Migrator {
        &SEEDED
    }

    fn rewrite(pending: Version) -> Option<Seeding> {
        StreamSchema::rewrite(pending)
            .map(Seeding::Streamed)
            .or_else(|| (pending == Seeding::UNTRIMMED).then_some(Seeding::Untrimmed))
    }
}

impl Batch for Seeding {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Seeding::Untrimmed => Around::untrimmed(statements),
            Seeding::Streamed(streamed) => streamed.written(statements),
        }
    }
}

static OLD_STORE: LazyLock<Sqlite> = LazyLock::new(Sqlite::in_memory::<Seeded>);

#[test]
fn a_segment_stored_before_stemming_is_found_by_its_stem() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(OLD_STORE.clone());
    let terms = Terms::lexed(["ban", "dyn"]).ok_or("no terms")?;
    let hits = store.read(SearchSegments::matching(terms, Scope::Owner))?;
    let found: Vec<String> = hits.iter().map(|hit| Body::from(hit).to_string()).collect();
    assert_eq!(found, ["banning dyn seems desirable"]);
    Ok(())
}

static BLANKS: LazyLock<Vec<char>> = LazyLock::new(|| {
    (char::MIN..=char::MAX)
        .filter(|point| Lexemes::scanned(&point.to_string()).iter().all(|lexeme| lexeme.ink().is_none()))
        .collect()
});

#[derive(Clone, Copy, Debug)]
struct Around(char);

impl Around {
    fn untrimmed(statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let written = BLANKS
            .iter()
            .map(|blank| Around(*blank).stored(statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }

    fn stored(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let uuid = self.uuid().map_err(|failed| StoreFailure::Refused(Box::new(failed)))?;
        let body = self.kept().map_err(|failed| StoreFailure::Refused(Box::new(failed)))?;
        let message = statements.written(store_ports::query!(
            r#"
INSERT INTO
  messages (uuid, session, parent, at, author, body)
VALUES
  (
    ?1,
    's',
    NULL,
    '2026-01-01T00:00:00Z',
    'owner',
    ?2
  )
"#,
            uuid: Uuid,
            body: Body
        ))?;
        let segment = statements.written(store_ports::query!(
            r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  (?1, 0, 'sentence', ?2)
"#,
            uuid: Uuid,
            body: Body
        ))?;
        Ok(message.plus(segment))
    }

    fn uuid(self) -> Result<Uuid, <Uuid as Checked>::Error> {
        Uuid::try_new(format!("{:08x}-0000-4000-8000-000000000000", u32::from(self.0)))
    }

    fn escaped(self) -> String {
        self.0.encode_utf16(&mut [0; 2]).iter().map(|unit| format!("\\u{unit:04x}")).collect::<Vec<String>>().concat()
    }

    fn session(self) -> Result<String, Box<dyn Error>> {
        Ok(format!(
            "{{\"type\":\"user\",\"uuid\":\"{}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{{\"kind\":\"human\"}},\"message\":{{\"content\":\"{}w{}\"}}}}\n",
            self.uuid()?,
            self.escaped(),
            self.escaped()
        ))
    }

    fn migrated(self) -> Result<Vec<String>, Box<dyn Error>> {
        Listed(self.uuid()?).shown(&Run::root().port(OLD_STORE.clone()))
    }

    fn fresh(self) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let fresh = run.port(Sqlite::in_memory::<StreamSchema>());
        fresh.write(fresh.read(Holding::from(Recorded::of(&self.session()?)?.records))?)?;
        Listed(self.uuid()?).shown(&fresh)
    }

    fn rederived(self) -> Result<Vec<Body>, Box<dyn Error + Send + Sync>> {
        let message = Message::from(Skipped::new(self.uuid()?, None).said(
            SessionId::try_new("s".to_owned())?,
            Instant::try_new("2026-01-01T00:00:00Z".to_owned())?,
            Authorship::Owner,
            self.kept()?,
        ));
        Ok(message.segments().iter().map(|segment| segment.text(&message)).collect::<Result<Vec<Body>, _>>()?)
    }

    fn kept(self) -> Result<Body, Nul> {
        Body::try_new(format!("{}w{}", self.0, self.0))
    }
}

struct Listed(Uuid);

impl Listed {
    fn shown(&self, store: &Port<'_, Sqlite>) -> Result<Vec<String>, Box<dyn Error>> {
        let shown = store.read(ShownMessage::named(self.0.clone()))?.ok_or("the message is not stored")?;
        Ok(shown
            .segments()
            .iter()
            .map(|segment| {
                format!("{} {} {:?}", segment.ordinal(), AsRef::<ShapeName>::as_ref(segment), Body::from(segment))
            })
            .collect())
    }

    fn found(words: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(OLD_STORE.clone());
        let terms = Terms::lexed(words.iter().copied()).ok_or("no terms")?;
        let hits = store.read(SearchSegments::matching(terms, Scope::Owner))?;
        Ok(hits.iter().map(|hit| Body::from(hit).to_string()).collect())
    }
}

static UNANCHORED_SEEDS: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!("tests/unanchored"));

static UNANCHORED: LazyLock<Migrator> = LazyLock::new(|| {
    Migrator::from(StreamSchema::migrator().iter().chain(UNANCHORED_SEEDS.iter()).copied().collect::<Vec<_>>())
});

#[derive(Clone, Copy, Debug)]
struct Unanchored;

impl Schema for Unanchored {
    type Rewrite = <StreamSchema as Schema>::Rewrite;

    fn migrator() -> &'static Migrator {
        &UNANCHORED
    }

    fn rewrite(pending: Version) -> Option<Self::Rewrite> {
        StreamSchema::rewrite(pending)
    }
}

impl Unanchored {
    fn refusal<Rows: std::fmt::Debug>(opened: Result<Rows, StoreFailure>) -> String {
        match opened {
            Err(StoreFailure::Refused(refusal)) => refusal.to_string(),
            other => format!("not refused: {other:?}"),
        }
    }
}

#[test]
fn an_old_store_holding_a_link_to_a_segment_it_does_not_hold_refuses_the_open_and_names_the_link() {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<Unanchored>());
    assert_eq!(
        Unanchored::refusal(store.read(SessionCursors)),
        "these stored links name a segment the store does not hold: u1#0 answers u1#1; u1#2 answers u1#3"
    );
}

#[derive(Newtype, Display, Clone, Debug, PartialEq, Eq)]
struct Named(String);

#[derive(Debug, Columns)]
struct SegmentKey {
    child: Named,
    on_update: Named,
    on_delete: Named,
    declared: Named,
}

#[derive(Debug, Op)]
struct SegmentKeys;

impl SegmentKey {
    fn kept(&self) -> String {
        let deferrable = self.declared.clone().into_inner().contains("DEFERRABLE");
        format!("{}: on update {}, on delete {}, deferrable {deferrable}", self.child, self.on_update, self.on_delete)
    }

    fn immediate(&self) -> String {
        format!("{}: on update NO ACTION, on delete NO ACTION, deferrable false", self.child)
    }
}

impl Query for SegmentKeys {
    type Rows = Vec<SegmentKey>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<SegmentKey>, StoreFailure> {
        statements.rows(store_ports::query!(
            SegmentKey,
            r#"
SELECT
  tables.name AS "child!: String",
  CAST(keys.on_update AS TEXT) AS "on_update!: String",
  CAST(keys.on_delete AS TEXT) AS "on_delete!: String",
  upper(tables.sql) AS "declared!: String"
FROM
  sqlite_schema AS tables
  JOIN pragma_foreign_key_list(tables.name) AS keys
WHERE
  tables.type = 'table'
  AND keys."table" = 'segments'
ORDER BY
  tables.name,
  keys.id
"#
        ))
    }
}

#[test]
fn every_foreign_key_into_segments_is_immediate_and_neither_cascades_nor_sets() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let keys = run.port(Sqlite::in_memory::<StreamSchema>()).read(SegmentKeys)?;
    let children: BTreeSet<String> = keys.iter().map(|key| key.child.clone().into_inner()).collect();
    assert!(children.contains("mechanical_links"), "{children:?}");
    let kept: Vec<String> = keys.iter().map(SegmentKey::kept).collect();
    let immediate: Vec<String> = keys.iter().map(SegmentKey::immediate).collect();
    assert_eq!(kept, immediate);
    Ok(())
}

const RESEGMENTED: &str = concat!(
    "{\"type\":\"user\",\"uuid\":\"o3\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:03Z\",\"origin\":{\"kind\":\"human\"},",
    "\"message\":{\"content\":\"First point. Second point.\"}}\n",
    "{\"type\":\"user\",\"uuid\":\"o4\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:04Z\",\"origin\":{\"kind\":\"human\"},",
    "\"message\":{\"content\":\"Only one.\"}}\n"
);

#[test]
fn a_message_stored_with_segments_or_shapes_it_no_longer_derives_holds_what_a_fresh_ingest_stores()
-> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let fresh = run.port(Sqlite::in_memory::<StreamSchema>());
    fresh.write(fresh.read(Holding::from(Recorded::of(RESEGMENTED)?.records))?)?;
    let old = run.port(OLD_STORE.clone());
    let listed = [Listed(Uuid::try_new("o3".to_owned())?), Listed(Uuid::try_new("o4".to_owned())?)];
    let migrated = listed.iter().map(|message| message.shown(&old)).collect::<Result<Vec<_>, _>>()?;
    let ingested = listed.iter().map(|message| message.shown(&fresh)).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(migrated, ingested);
    let resegmented = [
        vec!["0 sentence Body(\"First point.\")", "1 sentence Body(\"Second point.\")"],
        vec!["0 sentence Body(\"Only one.\")"],
    ];
    assert_eq!(migrated, resegmented);
    assert_eq!(Listed::found(&["second"])?, ["Second point."]);
    assert_eq!(Listed::found(&["astray"])?, Vec::<String>::new());
    Ok(())
}

#[test]
fn every_blank_the_segmenter_sheds_around_a_segment_stored_before_trimming_is_shed_as_a_fresh_ingest_sheds_it()
-> Result<(), Box<dyn Error>> {
    let blanks: Vec<Around> = BLANKS.iter().copied().map(Around).collect();
    let migrated = blanks.iter().map(|blank| blank.migrated()).collect::<Result<Vec<_>, _>>()?;
    let fresh = blanks.iter().map(|blank| blank.fresh()).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(migrated, fresh);
    Ok(())
}

#[test]
fn every_point_the_segmenter_inks_around_a_segment_stored_before_trimming_is_kept_on_both_sides()
-> Result<(), Box<dyn Error + Send + Sync>> {
    let inked: Vec<Around> = (char::MIN..=char::MAX)
        .filter(|point| BLANKS.binary_search(point).is_err())
        .filter(|point| Body::try_new(point.to_string()).is_ok())
        .map(Around)
        .collect();
    let rederived = inked.iter().map(|point| point.rederived()).collect::<Result<Vec<Vec<Body>>, _>>()?;
    let both_sides =
        inked.iter().map(|point| point.kept().map(|kept| vec![kept])).collect::<Result<Vec<Vec<Body>>, _>>()?;
    assert_eq!(rederived, both_sides);
    Ok(())
}

#[test]
fn a_segment_stored_with_a_body_its_message_no_longer_derives_is_rederived_and_found_by_its_words()
-> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(OLD_STORE.clone());
    let shown = store.read(ShownMessage::named(Uuid::try_new("o2".to_owned())?))?.ok_or("o2 was not stored")?;
    let bodies: Vec<String> = shown.segments().iter().map(|segment| Body::from(segment).to_string()).collect();
    assert_eq!(bodies, ["kept forever"]);
    let terms = Terms::lexed(["forever"]).ok_or("no terms")?;
    let hits = store.read(SearchSegments::matching(terms, Scope::Owner))?;
    let found: Vec<String> = hits.iter().map(|hit| Body::from(hit).to_string()).collect();
    assert_eq!(found, ["kept forever"]);
    Ok(())
}

#[derive(Clone, Debug)]
struct Dag {
    facets: Vec<Facet>,
    edges: Vec<Edge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    child: usize,
    parent: usize,
}

impl Dag {
    fn any() -> impl Strategy<Value = Dag> {
        vec((vec(any::<Index>(), 0..4), select(Facet::VARIANTS)), 1..16).prop_flat_map(|choices| {
            let facets: Vec<Facet> = choices.iter().map(|(_parents, facet)| *facet).collect();
            let edges: BTreeSet<Edge> = choices
                .iter()
                .enumerate()
                .skip(1)
                .flat_map(|(child, (parents, _facet))| {
                    parents.iter().map(move |index| Edge { child, parent: index.index(child) })
                })
                .collect();
            Just(edges.into_iter().collect::<Vec<Edge>>())
                .prop_shuffle()
                .prop_map(move |shuffled| Dag { facets: facets.clone(), edges: shuffled })
        })
    }

    fn id(node: usize) -> NodeId {
        NodeId::new(i64::try_from(node).unwrap_or(i64::MAX).saturating_add(1))
    }

    fn node(store: &Port<'_, Sqlite>, facet: Facet) -> Result<Planting, Box<dyn Error>> {
        Dag::planting(store, Entry::new("p".parse()?, facet, Provenance::from(Entered::Init)))
    }

    fn planting(store: &Port<'_, Sqlite>, entry: Entry) -> Result<Planting, Box<dyn Error>> {
        Ok(Planting::sown(Node::new(store.read(NextNode)?, entry)))
    }

    fn resolved(store: &Port<'_, Sqlite>, id: NodeId) -> Result<Resolved, Box<dyn Error>> {
        Ok(store.read(Resolving::from(id))?.ok_or("the node is not stored")?)
    }

    fn linked(
        store: &Port<'_, Sqlite>,
        [source, target]: [NodeId; 2],
        relation: NodeRelation,
    ) -> Result<Result<Written, Refusal>, Box<dyn Error>> {
        let (from, to) = (Dag::resolved(store, source)?, Dag::resolved(store, target)?);
        Ok(store.write(Linked::new(from, relation, to, Entered::Init.into())))
    }

    fn cast(store: &Port<'_, Sqlite>, node: NodeId, role: Role) -> Result<Result<Written, Refusal>, Box<dyn Error>> {
        let resolved = Dag::resolved(store, node)?;
        Ok(store.write(Cast::new(resolved, Casting::new(role, Entered::Role.into()))))
    }

    fn planted(&self, store: &Port<'_, Sqlite>) -> Result<Written, Box<dyn Error>> {
        let nodes = self
            .facets
            .iter()
            .map(|facet| Ok(store.write(Dag::node(store, *facet)?)?))
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        let links = self
            .edges
            .iter()
            .map(|edge| Ok(Dag::linked(store, [Dag::id(edge.child), Dag::id(edge.parent)], NodeRelation::Refines)??))
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        Ok(Written::total(nodes.into_iter().chain(links)))
    }

    fn parents(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.edges.iter().filter(move |edge| edge.child == node).map(|edge| edge.parent)
    }

    fn ancestors(&self, node: usize) -> Vec<NodeId> {
        let levels = std::iter::successors(Some(BTreeSet::from([node])), |frontier| {
            Some(frontier.iter().flat_map(|below| self.parents(*below)).collect::<BTreeSet<usize>>())
        })
        .take(self.facets.len());
        let depths = levels
            .enumerate()
            .skip(1)
            .flat_map(|(depth, level)| level.into_iter().map(move |above| (above, depth)))
            .fold(BTreeMap::new(), |mut depths, (above, depth)| {
                depths.entry(above).or_insert(depth);
                depths
            });
        let ordered: BTreeSet<(Reverse<usize>, usize)> =
            depths.into_iter().map(|(above, depth)| (Reverse(depth), above)).collect();
        ordered.into_iter().map(|(_depth, above)| Dag::id(above)).collect()
    }

    fn children(&self, node: usize) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|edge| edge.parent == node)
            .map(|edge| edge.child)
            .collect::<BTreeSet<usize>>()
            .into_iter()
            .map(Dag::id)
            .collect()
    }
}

struct Ids;

impl Ids {
    fn kin(placed: &Placed, kin: Kin) -> Vec<NodeId> {
        placed
            .relatives()
            .iter()
            .filter(|relative| relative.kin() == kin)
            .map(|relative| relative.node().identity())
            .collect()
    }

    fn earlier(seen: &Seen) -> Option<i64> {
        match seen {
            Seen::Before(earlier) => Some(earlier.identity().into_inner()),
            Seen::Unseen(_) => None,
        }
    }
}

laws::law!(a_node_s_kin_are_exactly_its_transitive_refinements_whatever_else_is_closed, Mixed::any(), mixed, {
    let shown = mixed.shown().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(shown, mixed.wanted());
});

struct Projects;

impl Projects {
    fn rooted(store: &Port<'_, Sqlite>, name: &str) -> Result<NodeId, Box<dyn Error>> {
        let project: Project = name.parse()?;
        store.write(Dag::planting(store, Entry::new(project.clone(), Facet::Project, Entered::Init.into()))?)?;
        Ok(store.read(ProjectRoot::from(project))?.ok_or("no root")?.identity())
    }
}

laws::law!(
    a_project_root_is_found_by_its_project_alone,
    select(Facet::VARIANTS).prop_filter("a facet other than a project root", |facet| *facet != Facet::Project),
    facet,
    {
        let found = Projects::alone(facet).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        prop_assert_eq!(found, [Some(NodeId::new(3)), None]);
    }
);

impl Projects {
    fn alone(facet: Facet) -> Result<[Option<NodeId>; 2], Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        Projects::rooted(&store, "q")?;
        store.write(Dag::node(&store, facet)?)?;
        store.write(Dag::planting(&store, Entry::new("p".parse()?, Facet::Project, Entered::Init.into()))?)?;
        Ok([
            store.read(ProjectRoot::from("p".parse::<Project>()?))?.map(Resolved::identity),
            store.read(ProjectRoot::from("r".parse::<Project>()?))?.map(Resolved::identity),
        ])
    }
}

#[test]
fn a_consumes_link_is_listed_from_its_source_and_adds_no_ancestor() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let eater = Projects::rooted(&store, "p")?;
    let consumed = Projects::rooted(&store, "q")?;
    Dag::linked(&store, [eater, consumed], NodeRelation::Consumes)??;
    let shown = store.read(ShownNode::from(eater))?.ok_or("no consumer shown")?;
    let links: Vec<NodeEdge> = shown.links().iter().map(|link| link.edge()).collect();
    assert_eq!(links, [NodeEdge::new(eater, NodeRelation::Consumes, consumed)]);
    assert_eq!(Ids::kin(&shown, Kin::Ancestor), []);
    assert_eq!(Ids::kin(&store.read(ShownNode::from(consumed))?.ok_or("no consumed shown")?, Kin::Child), []);
    Ok(())
}

#[test]
fn linking_the_same_nodes_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let parent = Projects::rooted(&store, "p")?;
    store.write(Dag::node(&store, Facet::Project)?)?;
    let child = NodeId::new(parent.into_inner().saturating_add(1));
    let first = Dag::linked(&store, [child, parent], NodeRelation::Refines)??;
    let second = Dag::linked(&store, [child, parent], NodeRelation::Refines)??;
    assert!(first > Written::zero());
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn a_shortcut_refinement_is_listed_as_a_child() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let edges = [(1, 0), (2, 1), (2, 0)].map(|(child, parent)| Edge { child, parent });
    Dag { facets: vec![Facet::Project; 3], edges: edges.to_vec() }.planted(&store)?;
    let shown = store.read(ShownNode::from(Dag::id(0)))?.ok_or("no root shown")?;
    assert_eq!(Ids::kin(&shown, Kin::Child), [Dag::id(1), Dag::id(2)]);
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

struct Proposals;

impl Proposals {
    fn node(id: i64) -> Result<Node<Vague>, Box<dyn Error>> {
        let entry = Entry::new("p".parse()?, Facet::Feature, Entered::Apply.into());
        Ok(Node::new(NodeId::new(id), entry).titled(Title::new(format!("child {id}"))))
    }

    fn child(parent: NodeId, id: i64) -> Result<Directed, Box<dyn Error>> {
        Ok(Directed::Proposed { node: Proposals::node(id)?, parent })
    }

    fn superseded(old: i64, by: i64) -> Result<Directed, Box<dyn Error>> {
        Ok(Directed::Superseded(Supersession::within(&Proposals::node(old)?, &Proposals::node(by)?)?))
    }

    fn written(store: &Port<'_, Sqlite>, directed: Vec<Directed>) -> Result<Written, Box<dyn Error>> {
        let file = Proposals::unseen(store, &format!("{directed:?}"))?;
        Ok(store.write(Directions::new(Provenance::from(Entered::Apply), file, directed, Remit::Unbounded))?)
    }

    fn unseen(store: &Port<'_, Sqlite>, contents: &str) -> Result<Unseen, Box<dyn Error>> {
        match store.read(AppliedFile::from(Bytes::from(contents)))? {
            Seen::Unseen(unseen) => Ok(unseen),
            Seen::Before(earlier) => {
                Err(format!("{contents} was applied as file {}", earlier.identity().into_inner()).into())
            }
        }
    }

    fn segment(message: &str, ordinal: usize) -> Result<Segment, Box<dyn Error>> {
        Ok(Segment::new(Uuid::try_new(message.to_owned())?, Ordinal::new(ordinal)))
    }
}

#[test]
fn a_proposal_plants_a_titled_vague_child_under_its_parent() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let next = store.read(NextNode)?;
    Proposals::written(&store, vec![Proposals::child(root, next.into_inner())?])?;
    let child = store.read(ShownNode::from(next))?.ok_or("the proposal was not planted")?;
    assert_eq!(child.node().stage().stage(), Stage::Vague);
    assert_eq!(child.node().title(), Some(&Title::new("child 2".to_owned())));
    assert_eq!(Ids::kin(&child, Kin::Ancestor), [root]);
    assert_eq!(Ids::kin(&store.read(ShownNode::from(root))?.ok_or("no root")?, Kin::Child), [next]);
    Ok(())
}

#[test]
fn the_next_node_follows_the_highest_node() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    assert_eq!(store.read(NextNode)?, NodeId::new(1));
    Projects::rooted(&store, "p")?;
    Projects::rooted(&store, "q")?;
    assert_eq!(store.read(NextNode)?, NodeId::new(3));
    Ok(())
}

#[test]
fn directions_are_written_whole_or_not_at_all() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(records))?)?;
    let root = Projects::rooted(&store, "p")?;
    let stray = Directed::Conversational { segment: Proposals::segment("nowhere", 0)? };
    assert!(Proposals::written(&store, vec![Proposals::child(root, 2)?, stray]).is_err());
    assert!(store.read(ShownNode::from(NodeId::new(2)))?.is_none());
    Ok(())
}

#[test]
fn a_file_is_seen_once_its_directions_are_written_and_not_when_they_are_refused() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(records))?)?;
    let root = Projects::rooted(&store, "p")?;
    let stray = Directed::Conversational { segment: Proposals::segment("nowhere", 0)? };
    let refused = Proposals::unseen(&store, "refused")?;
    assert!(
        store.write(Directions::new(Provenance::from(Entered::Apply), refused, vec![stray], Remit::Unbounded)).is_err()
    );
    let [asked, answered] = [Proposals::segment("o1", 0)?, Proposals::segment("o1", 1)?];
    let [first, second] = [JobId::new(1), JobId::new(2)];
    store.write(JobOpened::from(first))?;
    store.write(JobOpened::from(second))?;
    let job = Remit::job(first, [asked.clone(), answered.clone()]);
    let unpaid = Proposals::unseen(&store, "unpaid")?;
    let directed = vec![Proposals::child(root, 2)?, Directed::Conversational { segment: asked.clone() }];
    let refusal = store.write(Directions::new(Provenance::from(Entered::Apply), unpaid, directed.clone(), job)).err();
    assert!(matches!(refusal, Some(Refusal::Unpaid(_))));
    assert_eq!(store.read(OpenJobs)?, [first, second]);
    let kept = Proposals::unseen(&store, "kept")?;
    store.write(Directions::new(
        Provenance::from(Entered::Apply),
        kept,
        directed,
        Remit::job(first, [asked.clone()]),
    ))?;
    assert!(matches!(store.write(JobRepeated::from(Remit::job(second, [answered]))), Err(Refusal::Unpaid(_))));
    assert_eq!(store.read(OpenJobs)?, [second]);
    store.write(JobRepeated::from(Remit::job(second, [asked])))?;
    assert_eq!(store.read(OpenJobs)?, []);
    let other = Proposals::unseen(&store, "other")?;
    store.write(Directions::new(Provenance::from(Entered::Apply), other, Vec::new(), Remit::Unbounded))?;
    let seen = [
        Bytes::from("kept"),
        Bytes::from("other"),
        Bytes::from("kept "),
        Bytes::from("refused"),
        Bytes::from("unpaid"),
    ]
    .map(|contents| store.read(AppliedFile::from(contents)).map(|seen| Ids::earlier(&seen)))
    .into_iter()
    .collect::<Result<Vec<Option<i64>>, _>>()?;
    assert_eq!(seen, [Some(1), Some(2), None, None, None]);
    Ok(())
}

#[test]
fn a_superseded_node_stays_shown_and_its_successor_links_to_it() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    Proposals::written(&store, vec![Proposals::child(root, 2)?, Proposals::child(root, 3)?])?;
    Proposals::written(&store, vec![Proposals::superseded(2, 3)?])?;
    let shown = store.read(ShownNode::from(NodeId::new(3)))?.ok_or("no successor")?;
    let links: Vec<(NodeEdge, Provenance)> =
        shown.links().iter().map(|link| (link.edge(), link.provenance())).collect();
    let applied = Provenance::from(Entered::Apply);
    let successor = NodeId::new(3);
    assert_eq!(
        links,
        [
            (NodeEdge::new(successor, NodeRelation::Refines, root), applied),
            (NodeEdge::new(successor, NodeRelation::Supersedes, NodeId::new(2)), applied)
        ]
    );
    assert!(store.read(ShownNode::from(NodeId::new(2)))?.is_some());
    Ok(())
}

#[test]
fn a_prefix_finds_every_segment_of_the_messages_it_starts() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(records))?)?;
    let heard = |prefix: &str| -> Result<Vec<(Segment, Authorship)>, Box<dyn Error>> {
        let spoken = store.read(Prefixed::from(Prefix::new(prefix.to_owned())))?;
        Ok(spoken.iter().map(|said| (said.segment().clone(), said.author())).collect())
    };
    let owner = [(Proposals::segment("o1", 0)?, Authorship::Owner), (Proposals::segment("o1", 1)?, Authorship::Owner)];
    assert_eq!(heard("o")?, owner);
    assert_eq!(heard("o1")?, owner);
    assert_eq!(heard("l1")?, [(Proposals::segment("l1", 0)?, Authorship::Lead)]);
    assert_eq!(heard("1")?, []);
    Ok(())
}

#[test]
fn a_segment_is_covered_by_its_own_directive_not_by_a_link_to_it() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(records))?)?;
    let [asked, answered, said] =
        [Proposals::segment("o1", 0)?, Proposals::segment("o1", 1)?, Proposals::segment("l1", 0)?];
    let covered = |messages: &[&str]| -> Result<Vec<Segment>, Box<dyn Error>> {
        let uuids = messages
            .iter()
            .map(|message| Ok(Uuid::try_new((*message).to_owned())?))
            .collect::<Result<Vec<Uuid>, Box<dyn Error>>>()?;
        Ok(store.read(uuids.into_iter().collect::<Covered>())?.into_iter().collect())
    };
    assert_eq!(covered(&["o1", "l1"])?, []);
    let related = Directed::Related { source: said.clone(), target: answered, relation: SegmentRelation::Answers };
    Proposals::written(&store, vec![Directed::Conversational { segment: asked.clone() }, related])?;
    assert_eq!(covered(&["o1"])?, std::slice::from_ref(&asked));
    assert_eq!(covered(&["l1"])?, std::slice::from_ref(&said));
    assert_eq!(covered(&["o1", "l1"])?, [said, asked]);
    Ok(())
}

const SEGMENTED: &str = concat!(
    "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},",
    "\"message\":{\"content\":\"1. a\\n2. b\\n3. c\\n4. d\\n5. e\"}}\n"
);

const ENDS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closed {
    Refines,
    Supersedes,
    SegmentSupersedes,
    SegmentCorrects,
}

#[derive(Clone, Copy, Debug)]
enum Ends {
    Nodes(NodeRelation),
    Segments(SegmentRelation),
}

#[derive(Clone, Debug)]
struct Probe {
    closed: Closed,
    edges: Vec<Edge>,
    tried: Edge,
}

impl Closed {
    const ALL: [Closed; 4] = [Closed::Refines, Closed::Supersedes, Closed::SegmentSupersedes, Closed::SegmentCorrects];

    fn ends(self) -> Ends {
        match self {
            Closed::Refines => Ends::Nodes(NodeRelation::Refines),
            Closed::Supersedes => Ends::Nodes(NodeRelation::Supersedes),
            Closed::SegmentSupersedes => Ends::Segments(SegmentRelation::Supersedes),
            Closed::SegmentCorrects => Ends::Segments(SegmentRelation::Corrects),
        }
    }
}

impl Ends {
    fn linked(
        self,
        store: &Port<'_, Sqlite>,
        edge: Edge,
        file: Unseen,
    ) -> Result<Result<Written, Refusal>, Box<dyn Error>> {
        match self {
            Ends::Nodes(relation) => Dag::linked(store, [Dag::id(edge.child), Dag::id(edge.parent)], relation),
            Ends::Segments(relation) => Ok(store.write(Directions::new(
                Provenance::from(Entered::Apply),
                file,
                vec![Directed::Related {
                    source: Proposals::segment("o1", edge.child)?,
                    target: Proposals::segment("o1", edge.parent)?,
                    relation,
                }],
                Remit::Unbounded,
            ))),
        }
    }

    fn cycle(self, edge: Edge) -> Result<Cycle, Box<dyn Error>> {
        match self {
            Ends::Nodes(relation) => {
                Ok(Cycle::Nodes { source: Dag::id(edge.child), relation, target: Dag::id(edge.parent) })
            }
            Ends::Segments(relation) => Ok(Cycle::Segments {
                source: Proposals::segment("o1", edge.child)?,
                relation,
                target: Proposals::segment("o1", edge.parent)?,
            }),
        }
    }
}

impl Probe {
    fn any() -> impl Strategy<Value = Probe> {
        (select(Closed::ALL.to_vec()), Probe::edges(), 0..ENDS, 0..ENDS)
            .prop_map(|(closed, edges, child, parent)| Probe { closed, edges, tried: Edge { child, parent } })
    }

    fn edges() -> impl Strategy<Value = Vec<Edge>> {
        vec((0..ENDS, 0..ENDS), 0..10).prop_map(|pairs| {
            pairs
                .into_iter()
                .filter(|(first, second)| first != second)
                .map(|(first, second)| Edge { child: first.max(second), parent: first.min(second) })
                .collect()
        })
    }

    fn refused(&self) -> Result<Option<Cycle>, Box<dyn Error>> {
        let Recorded { records, .. } = Recorded::of(SEGMENTED)?;
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(store.read(Holding::from(records))?)?;
        iter::repeat_with(|| Ok(store.write(Dag::node(&store, Facet::Feature)?)?))
            .take(ENDS)
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        let ends = self.closed.ends();
        let mut files = (0..=self.edges.len())
            .map(|nth| Proposals::unseen(&store, &nth.to_string()))
            .collect::<Result<Vec<Unseen>, Box<dyn Error>>>()?
            .into_iter();
        let tried = files.next().ok_or("no file for the tried edge")?;
        self.edges
            .iter()
            .zip(files)
            .map(|(edge, file)| Ok(ends.linked(&store, *edge, file)??))
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        Probe::cyclic(ends.linked(&store, self.tried, tried)?)
    }

    fn cyclic(written: Result<Written, Refusal>) -> Result<Option<Cycle>, Box<dyn Error>> {
        match written {
            Ok(_written) => Ok(None),
            Err(Refusal::Cyclic(cycle)) => Ok(Some(cycle)),
            Err(failure) => Err(failure.into()),
        }
    }

    fn reached(&self) -> BTreeSet<usize> {
        std::iter::successors(Some(BTreeSet::from([self.tried.parent])), |frontier| {
            let next: BTreeSet<usize> =
                self.edges.iter().filter(|edge| frontier.contains(&edge.child)).map(|edge| edge.parent).collect();
            Some(frontier.union(&next).copied().collect())
        })
        .nth(ENDS)
        .unwrap_or_default()
    }

    fn chain() -> Vec<Edge> {
        vec![Edge { child: 1, parent: 0 }, Edge { child: 2, parent: 1 }]
    }

    fn closing() -> Edge {
        Edge { child: 0, parent: 2 }
    }

    fn wanted(&self) -> Result<Option<Cycle>, Box<dyn Error>> {
        self.reached().get(&self.tried.child).map(|_reached| self.closed.ends().cycle(self.tried)).transpose()
    }
}

laws::law!(an_edge_is_refused_exactly_where_the_closure_holds_its_reverse_path, Probe::any(), probe, {
    let refused = probe.refused().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(refused, probe.wanted().map_err(|failure| TestCaseError::fail(failure.to_string()))?);
});

#[test]
fn an_edge_reversing_a_written_path_is_refused_for_every_acyclic_relation() -> Result<(), Box<dyn Error>> {
    let refused = Closed::ALL
        .into_iter()
        .map(|closed| Probe { closed, edges: Probe::chain(), tried: Probe::closing() }.refused())
        .collect::<Result<Vec<Option<Cycle>>, Box<dyn Error>>>()?;
    let cycles = Closed::ALL
        .into_iter()
        .map(|closed| Ok(Some(closed.ends().cycle(Probe::closing())?)))
        .collect::<Result<Vec<Option<Cycle>>, Box<dyn Error>>>()?;
    assert_eq!(refused, cycles);
    Ok(())
}

#[derive(Clone, Debug)]
struct Mixed {
    dag: Dag,
    closed: Closed,
    edges: Vec<Edge>,
}

#[derive(Debug, PartialEq, Eq)]
struct Shown {
    ancestors: Vec<NodeId>,
    children: Vec<NodeId>,
    facet: Facet,
}

impl Mixed {
    fn any() -> impl Strategy<Value = Mixed> {
        let others: Vec<Closed> = Closed::ALL.into_iter().filter(|closed| *closed != Closed::Refines).collect();
        (Dag::any(), select(others), Probe::edges()).prop_map(|(dag, closed, edges)| {
            let ends = dag.facets.len().min(ENDS);
            Mixed { dag, closed, edges: edges.into_iter().filter(|edge| edge.child < ends).collect() }
        })
    }

    fn shown(&self) -> Result<Vec<Shown>, Box<dyn Error>> {
        let Recorded { records, .. } = Recorded::of(SEGMENTED)?;
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(store.read(Holding::from(records))?)?;
        self.dag.planted(&store)?;
        let ends = self.closed.ends();
        let files = (0..self.edges.len())
            .map(|nth| Proposals::unseen(&store, &nth.to_string()))
            .collect::<Result<Vec<Unseen>, Box<dyn Error>>>()?;
        self.edges
            .iter()
            .zip(files)
            .map(|(edge, file)| {
                Ok(ends.linked(&store, *edge, file)?.or_else(|refusal| {
                    matches!(refusal, Refusal::Ancestral { .. }).then_some(Written::zero()).ok_or(refusal)
                })?)
            })
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        (0..self.dag.facets.len())
            .map(|node| {
                let placed = store.read(ShownNode::from(Dag::id(node)))?.ok_or("a planted node is not shown")?;
                Ok(Shown {
                    ancestors: Ids::kin(&placed, Kin::Ancestor),
                    children: Ids::kin(&placed, Kin::Child),
                    facet: placed.node().entry().facet(),
                })
            })
            .collect()
    }

    fn wanted(&self) -> Vec<Shown> {
        self.dag
            .facets
            .iter()
            .enumerate()
            .map(|(node, facet)| Shown {
                ancestors: self.dag.ancestors(node),
                children: self.dag.children(node),
                facet: *facet,
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Restaging {
    refused: Option<Misstaged>,
    node: AnyNode,
}

#[derive(Debug, PartialEq, Eq)]
struct Mismatch {
    wanted: Restaging,
    stored: Restaging,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Refine,
    Specify,
    Reopen,
    Drop,
}

struct Staging;

impl Staging {
    const STEPS: [Step; 4] = [Step::Refine, Step::Specify, Step::Reopen, Step::Drop];

    fn transition(step: Step) -> Result<Transition, Box<dyn Error>> {
        match step {
            Step::Refine => Ok(Transition::Refine),
            Step::Specify => {
                Ok(Transition::Specify { artifact: "tool:design".parse()?, condition: "design show lists it".parse()? })
            }
            Step::Reopen => Ok(Transition::Reopen),
            Step::Drop => Ok(Transition::Drop { reason: "superseded by a smaller node".parse()? }),
        }
    }

    fn stored(store: &Port<'_, Sqlite>, id: NodeId) -> Result<AnyNode, Box<dyn Error>> {
        Ok(store.read(ShownNode::from(id))?.ok_or("the node is not shown")?.node().clone())
    }

    fn mismatched(steps: &[Step]) -> Result<Vec<Mismatch>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        let root = Projects::rooted(&store, "p")?;
        let resolved = Dag::resolved(&store, root)?;
        let moved = steps
            .iter()
            .map(|step| {
                let before = Staging::stored(&store, root)?;
                let transition = Staging::transition(*step)?;
                let wanted = transition.clone().applied(before.clone()).map_or_else(
                    |misstaged| Restaging { refused: Some(misstaged), node: before },
                    |moved| Restaging { refused: None, node: moved },
                );
                let refused =
                    store.write(Restaged::new(resolved, transition)).err().map(Staging::misstaged).transpose()?;
                Ok(Mismatch { wanted, stored: Restaging { refused, node: Staging::stored(&store, root)? } })
            })
            .collect::<Result<Vec<Mismatch>, Box<dyn Error>>>()?;
        Ok(moved.into_iter().filter(|mismatch| mismatch.wanted != mismatch.stored).collect())
    }

    fn misstaged(refusal: Refusal) -> Result<Misstaged, Box<dyn Error>> {
        match refusal {
            Refusal::Misstaged(misstaged) => Ok(misstaged),
            other @ (Refusal::Cyclic(_)
            | Refusal::Ancestral { .. }
            | Refusal::Inverted(_)
            | Refusal::Unpaid(_)
            | Refusal::Store(_)) => Err(other.into()),
        }
    }
}

laws::law!(
    a_restaged_node_reads_back_as_its_transition_applies_or_is_refused_unchanged_as_misstaged,
    vec(select(Staging::STEPS.to_vec()), 0..8),
    steps,
    {
        let mismatched = Staging::mismatched(&steps).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        prop_assert_eq!(mismatched, Vec::<Mismatch>::new());
    }
);

#[test]
fn a_dropped_node_stays_shown_under_its_parent() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    Proposals::written(&store, vec![Proposals::child(root, 2)?])?;
    store.write(Restaged::new(Dag::resolved(&store, NodeId::new(2))?, Staging::transition(Step::Drop)?))?;
    assert_eq!(Staging::stored(&store, NodeId::new(2))?.stage().stage(), Stage::Dropped);
    assert_eq!(Ids::kin(&store.read(ShownNode::from(root))?.ok_or("no root")?, Kin::Child), [NodeId::new(2)]);
    Ok(())
}

#[test]
fn a_vague_node_is_refused_a_specification_and_keeps_its_stage() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let specified = store.write(Restaged::new(Dag::resolved(&store, root)?, Staging::transition(Step::Specify)?));
    let refused = specified.err().map(Staging::misstaged).transpose()?;
    assert_eq!(refused.map(Misstaged::stage), Some(Stage::Vague));
    assert_eq!(Staging::stored(&store, root)?.stage().stage(), Stage::Vague);
    Ok(())
}

#[test]
fn a_missing_node_resolves_to_nothing_and_once_planted_refines_without_a_cycle() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let missing = NodeId::new(root.into_inner().saturating_add(1));
    assert_eq!(store.read(Resolving::from(missing))?, None);
    store.write(Dag::node(&store, Facet::Feature)?)?;
    assert!(Dag::linked(&store, [missing, root], NodeRelation::Refines)?? > Written::zero());
    Ok(())
}

#[test]
fn a_proposal_under_a_missing_parent_is_refused_and_closes_nothing() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let missing = NodeId::new(9);
    assert!(Proposals::written(&store, vec![Proposals::child(missing, 2)?]).is_err());
    assert_eq!(store.read(Resolving::from(NodeId::new(2)))?, None);
    Proposals::written(&store, vec![Proposals::child(root, 2)?])?;
    store.write(Planting::sown(Proposals::node(9)?))?;
    assert!(Dag::linked(&store, [missing, NodeId::new(2)], NodeRelation::Refines)?? > Written::zero());
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Change {
    Cast(usize, Role),
    Refined(Edge),
}

#[derive(Clone, Debug)]
struct Changes(Vec<Change>);

#[derive(Debug, PartialEq, Eq)]
struct Settled {
    refused: Vec<Option<NodeEdge>>,
    roles: Vec<Role>,
}

#[derive(Clone, Copy, Debug)]
struct Held {
    roles: [Role; ENDS],
}

impl Changes {
    fn any() -> impl Strategy<Value = Changes> {
        let cast = (0..ENDS, select(Role::VARIANTS)).prop_map(|(node, role)| Change::Cast(node, role));
        let refined = (0..ENDS, 0..ENDS)
            .prop_filter("two nodes", |(first, second)| first != second)
            .prop_map(|(first, second)| Change::Refined(Edge { child: first.max(second), parent: first.min(second) }));
        vec(prop_oneof![cast, refined], 0..14).prop_map(Changes)
    }

    fn written(&self) -> Result<Settled, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        iter::repeat_with(|| Ok(store.write(Dag::node(&store, Facet::Feature)?)?))
            .take(ENDS)
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        let refused = self
            .0
            .iter()
            .map(|change| change.written(&store).and_then(Changes::inverted))
            .collect::<Result<Vec<Option<NodeEdge>>, Box<dyn Error>>>()?;
        let roles = (0..ENDS)
            .map(|node| Ok(Staging::stored(&store, Dag::id(node))?.casting().role()))
            .collect::<Result<Vec<Role>, Box<dyn Error>>>()?;
        Ok(Settled { refused, roles })
    }

    fn inverted(written: Result<Written, Refusal>) -> Result<Option<NodeEdge>, Box<dyn Error>> {
        match written {
            Ok(_written) => Ok(None),
            Err(Refusal::Inverted(inversion)) => Ok(Some(inversion.edge())),
            Err(failure) => Err(failure.into()),
        }
    }

    fn wanted(&self) -> Settled {
        let start = Model { held: Held { roles: [Role::Core; ENDS] }, edges: BTreeSet::new(), refused: Vec::new() };
        let model = self.0.iter().fold(start, |model, change| model.after(*change));
        Settled { refused: model.refused, roles: model.held.roles.to_vec() }
    }
}

impl Change {
    fn written(self, store: &Port<'_, Sqlite>) -> Result<Result<Written, Refusal>, Box<dyn Error>> {
        match self {
            Change::Cast(node, role) => Dag::cast(store, Dag::id(node), role),
            Change::Refined(edge) => {
                Dag::linked(store, [Dag::id(edge.child), Dag::id(edge.parent)], NodeRelation::Refines)
            }
        }
    }

    fn touched(self) -> usize {
        match self {
            Change::Cast(node, _role) => node,
            Change::Refined(edge) => edge.child,
        }
    }
}

#[derive(Clone, Debug)]
struct Model {
    held: Held,
    edges: BTreeSet<Edge>,
    refused: Vec<Option<NodeEdge>>,
}

impl Model {
    fn after(self, change: Change) -> Model {
        let tried = self.clone().tried(change);
        let inverted = tried.inverted(change.touched());
        let kept = inverted.map_or(tried, |_refused| self);
        Model { refused: kept.refused.iter().copied().chain([inverted]).collect(), ..kept }
    }

    fn tried(self, change: Change) -> Model {
        match change {
            Change::Cast(node, role) => Model { held: self.held.cast(node, role), ..self },
            Change::Refined(edge) => Model { edges: self.edges.iter().copied().chain([edge]).collect(), ..self },
        }
    }

    fn inverted(&self, touched: usize) -> Option<NodeEdge> {
        self.edges
            .iter()
            .filter(|edge| [edge.child, edge.parent].contains(&touched))
            .find_map(|edge| self.held.inverted(*edge))
            .map(|edge| NodeEdge::new(Dag::id(edge.child), NodeRelation::Refines, Dag::id(edge.parent)))
    }
}

impl Held {
    fn cast(self, node: usize, role: Role) -> Held {
        let mut roles = self.roles;
        roles.get_mut(node).into_iter().for_each(|slot| *slot = role);
        Held { roles }
    }

    fn inverted(self, edge: Edge) -> Option<Edge> {
        let leaning = [self.roles.get(edge.child), self.roles.get(edge.parent)];
        (leaning == [Some(&Role::Core), Some(&Role::Secondary)]).then_some(edge)
    }
}

laws::law!(a_write_is_refused_exactly_where_a_core_node_would_depend_on_a_secondary_one, Changes::any(), leanings, {
    let written = leanings.written().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(written, leanings.wanted());
});

#[test]
fn a_core_project_cannot_consume_a_secondary_one() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let eating = Projects::rooted(&store, "p")?;
    let consumed = Projects::rooted(&store, "q")?;
    Dag::cast(&store, consumed, Role::Secondary)??;
    let refused = Changes::inverted(Dag::linked(&store, [eating, consumed], NodeRelation::Consumes)?)?;
    assert_eq!(refused, Some(NodeEdge::new(eating, NodeRelation::Consumes, consumed)));
    assert_eq!(store.read(ShownNode::from(eating))?.ok_or("no eating")?.links().len(), 0);
    Dag::cast(&store, eating, Role::Secondary)??;
    Dag::linked(&store, [eating, consumed], NodeRelation::Consumes)??;
    let inverted = Changes::inverted(Dag::cast(&store, eating, Role::Core)?)?;
    assert_eq!(inverted, Some(NodeEdge::new(eating, NodeRelation::Consumes, consumed)));
    Ok(())
}

struct Unstructuring;

impl Unstructuring {
    fn listed(store: &Port<'_, Sqlite>) -> Result<Vec<Segment>, Box<dyn Error>> {
        Ok(store.read(Unstructured)?.iter().map(|unplaced| AsRef::<Segment>::as_ref(unplaced).clone()).collect())
    }
}

#[test]
fn the_unstructured_owner_segments_shrink_as_directives_cover_them() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    let before = Unstructuring::listed(&store)?;
    Proposals::written(&store, vec![Directed::Conversational { segment: Proposals::segment("o1", 0)? }])?;
    let after = Unstructuring::listed(&store)?;
    assert_eq!(
        (before, after),
        (vec![Proposals::segment("o1", 0)?, Proposals::segment("o1", 1)?], vec![Proposals::segment("o1", 1)?])
    );
    Ok(())
}

#[test]
fn the_structurer_s_context_outlines_every_live_node_past_200_with_its_parent() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(Dag::planting(&store, Entry::new("q".parse()?, Facet::Project, Provenance::from(Entered::Init)))?)?;
    store.write(Dag::node(&store, Facet::Project)?)?;
    iter::repeat_with(|| Ok(store.write(Dag::node(&store, Facet::Feature)?)?))
        .take(201)
        .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
    let [specified, dropped] = [NodeId::new(203), NodeId::new(202)];
    [Step::Refine, Step::Specify]
        .into_iter()
        .map(|step| Ok(store.write(Restaged::new(Dag::resolved(&store, specified)?, Staging::transition(step)?))?))
        .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
    store.write(Restaged::new(Dag::resolved(&store, dropped)?, Staging::transition(Step::Drop)?))?;
    let child = Dag::node(&store, Facet::Feature)?;
    store.write(child)?;
    Dag::linked(&store, [NodeId::new(204), specified], NodeRelation::Refines)??;
    let context: Vec<(NodeId, Option<NodeId>)> =
        store.read(ContextNodes)?.iter().map(|outline| (outline.node(), outline.parent())).collect();
    let live = (2..=201).chain([203, 204, 1]).map(NodeId::new);
    let parents = live.map(|node| (node, (node == NodeId::new(204)).then_some(specified)));
    assert_eq!(context, parents.collect::<Vec<(NodeId, Option<NodeId>)>>());
    Ok(())
}

#[test]
fn a_flailed_mark_lands_only_on_unstructured_segments_is_listed_with_its_job_and_run_stays_unplaced_and_lifts_once_structured()
-> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    let job = store.read(NextJob)?;
    store.write(JobOpened::from(job))?;
    let spawned = Spawned::new(Pid::new(7), None, Deadline::new(Wall::new(0)));
    store.write(RunSpawned::new(RunId::new(1), job, spawned))?;
    let [structured, unstructured] = [Proposals::segment("o1", 0)?, Proposals::segment("o1", 1)?];
    Proposals::written(&store, vec![Directed::Conversational { segment: structured.clone() }])?;
    let held = Some(JobFlailed::new(RunId::new(1), vec![structured, unstructured.clone()]));
    store.write(JobHalted::new(job, Halted::Flailed, Account::told(&"flailed")).holding(held))?;
    let flailed: Vec<(Segment, RunId)> = store
        .read(FlailedSegments)?
        .iter()
        .filter(|flailed| flailed.job() == job)
        .map(|flailed| (AsRef::<Segment>::as_ref(flailed).clone(), flailed.run()))
        .collect();
    assert_eq!(Unstructuring::listed(&store)?, Vec::<Segment>::new());
    let unplaced = store.read(UnplacedSegments)?;
    assert_eq!(unplaced.iter().map(AsRef::<Segment>::as_ref).collect::<Vec<&Segment>>(), vec![&unstructured]);
    assert_eq!(flailed, vec![(unstructured.clone(), RunId::new(1))]);
    Proposals::written(&store, vec![Directed::Conversational { segment: unstructured }])?;
    assert_eq!(store.read(FlailedSegments)?, Vec::new());
    assert_eq!(store.read(UnplacedSegments)?, Vec::new());
    Ok(())
}

#[test]
fn a_retried_hold_withdraws_its_account_and_queues_its_segments_again() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    let job = store.read(NextJob)?;
    store.write(JobOpened::from(job))?;
    let spawned = Spawned::new(Pid::new(7), None, Deadline::new(Wall::new(0)));
    store.write(RunSpawned::new(RunId::new(1), job, spawned))?;
    let segments = vec![Proposals::segment("o1", 0)?, Proposals::segment("o1", 1)?];
    let flail = Some(JobFlailed::new(RunId::new(1), segments.clone()));
    store.write(JobHalted::new(job, Halted::Flailed, Account::told(&"flailed")).holding(flail))?;
    let queued = Unstructuring::listed(&store)?;
    let told = store.read(UnreportedFailures)?.len();
    let hold = store.read(HeldBy::from(job))?.ok_or("no hold")?;
    let holding: Vec<Segment> = hold.held().map(|flailed| AsRef::<Segment>::as_ref(flailed).clone()).collect();
    store.write(JobRetried::from(&hold))?;
    assert_eq!((queued, told, holding), (Vec::new(), 1, segments.clone()));
    assert_eq!(Unstructuring::listed(&store)?, segments);
    assert_eq!((store.read(HeldBy::from(job))?, store.read(UnreportedFailures)?), (None, Vec::new()));
    Ok(())
}

#[derive(Clone, Debug)]
struct Spawning {
    pid: Pid,
    started: Option<Started>,
    deadline: Wall,
    state: RunState,
    usage: Option<Usage>,
    reply: Option<Reply>,
}

impl Spawning {
    fn any() -> impl Strategy<Value = Spawning> {
        (
            any::<Pid>(),
            option::of(
                Just("Sat Nov  7 10:00:00 2026").prop_filter_map("a start time", |text| text.parse::<Started>().ok()),
            ),
            any::<Wall>(),
            select(RunState::VARIANTS),
            option::of((any::<Tokens>(), any::<Tokens>(), any::<Tokens>(), any::<Tokens>())),
            option::of(("[a-z\"\n€]{1,8}", 0..6000_usize).prop_map(|(unit, times)| Reply::new(unit.repeat(times)))),
        )
            .prop_map(|(pid, started, deadline, state, usage, reply)| Spawning {
                pid,
                started,
                deadline,
                state,
                usage: usage.map(|(input, output, written, read)| Usage::new(input, output, written, read)),
                reply,
            })
    }

    fn stored(&self) -> Result<Lived, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
        let job = store.read(NextJob)?;
        store.write(JobOpened::from(job))?;
        let id = store.read(NextRun)?;
        let spawned = Spawned::new(self.pid, self.started.clone(), Deadline::new(self.deadline));
        store.write(RunSpawned::new(id, job, spawned))?;
        let live = store.read(LiveRuns)?;
        let finished = self.reply.clone().map(|reply| Finished::new(reply, self.usage.unwrap_or_default()));
        store
            .write(finished.map_or(RunEnded::new(id, self.state, self.usage), |told| RunEnded::finished(id, &told)))?;
        Ok(Lived { live, ran: store.read(RunsOf::from(job))?, after: store.read(LiveRuns)? })
    }
}

#[derive(Debug)]
struct Lived {
    live: Vec<StoredRun>,
    ran: Vec<Ran>,
    after: Vec<StoredRun>,
}

laws::law!(
    a_spawned_run_is_live_until_it_ends_with_its_state_usage_and_reply_bounded_to_sixteen_kibibytes,
    Spawning::any(),
    spawning,
    {
        let lived = spawning.stored().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let spawned: Vec<Spawned> = lived.live.iter().map(|stored| stored.spawned().clone()).collect();
        let wanted = Spawned::new(spawning.pid, spawning.started.clone(), Deadline::new(spawning.deadline));
        prop_assert_eq!(spawned, vec![wanted]);
        let ran: Vec<Ran> = lived.ran;
        prop_assert_eq!(ran.iter().map(|one| [one.identity()]).collect::<Vec<[RunId; 1]>>(), vec![[RunId::new(1)]]);
        let replied = spawning.reply.as_ref().map(|_reply| RunState::Replied);
        let state = replied.unwrap_or(spawning.state);
        let usage = replied.map_or(spawning.usage, |_replied| Some(spawning.usage.unwrap_or_default()));
        prop_assert_eq!(ran.iter().map(Ran::state).collect::<Vec<RunState>>(), vec![state]);
        prop_assert_eq!(ran.iter().map(Ran::usage).collect::<Vec<Option<Usage>>>(), vec![usage]);
        let told = spawning.reply.map(Reply::into_inner);
        let kept = ran.first().and_then(Ran::reply).cloned().map(Excerpt::into_inner);
        prop_assert_eq!(kept.is_some(), told.is_some());
        if let (Some(whole), Some(part)) = (told, kept) {
            prop_assert!(part.len() <= whole.len());
            prop_assert!(iter::zip(part.bytes(), whole.bytes()).all(|(mine, theirs)| mine == theirs));
            prop_assert!(part.len() <= Excerpt::BYTES);
            let next = whole.ceil_char_boundary(part.len().saturating_add(1));
            prop_assert!(part.len() == whole.len() || next > Excerpt::BYTES);
            prop_assert!(whole.len() > Excerpt::BYTES || part == whole);
        }
        let still: Vec<RunId> = lived.after.iter().map(StoredRun::identity).collect();
        let running = [state].into_iter().filter(|now| *now == RunState::Running).map(|_| RunId::new(1));
        prop_assert_eq!(still, running.collect::<Vec<RunId>>());
    }
);

impl Spawning {
    fn replying(reply: String) -> Spawning {
        Spawning {
            pid: Pid::new(7),
            started: None,
            deadline: Wall::new(0),
            state: RunState::Replied,
            usage: None,
            reply: Some(Reply::new(reply)),
        }
    }

    fn kept(self) -> Result<String, Box<dyn Error>> {
        let lived = self.stored()?;
        let reply = lived.ran.first().and_then(Ran::reply).cloned().ok_or("no reply kept")?;
        Ok(Excerpt::into_inner(reply))
    }
}

#[test]
fn an_ascii_reply_of_twenty_thousand_bytes_is_stored_as_exactly_sixteen_kibibytes() -> Result<(), Box<dyn Error>> {
    let kept = Spawning::replying("a".repeat(20_000)).kept()?;
    assert_eq!(kept.len(), 16 * 1024);
    Ok(())
}

#[test]
fn an_ascii_reply_of_exactly_sixteen_kibibytes_is_stored_unchanged() -> Result<(), Box<dyn Error>> {
    let told = "a".repeat(16 * 1024);
    let kept = Spawning::replying(told.clone()).kept()?;
    assert_eq!(kept, told);
    Ok(())
}

#[test]
fn a_retold_account_of_an_already_reported_failure_is_reported_again() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let [first, second] = [JobId::new(1), JobId::new(2)];
    store.write(JobOpened::from(first))?;
    store.write(JobOpened::from(second))?;
    store.write(JobHalted::new(first, Halted::Flailed, Account::told(&"went on")))?;
    store.write([first].into_iter().collect::<FailuresReported>())?;
    let retold = vec![Retold::new(first, Account::told(&"went on, then halted"))];
    store.write(JobHalted::new(second, Halted::Stalled, Account::told(&"stalled")).retelling(retold))?;
    let told: Vec<String> =
        store.read(UnreportedFailures)?.iter().map(|failure| AsRef::<Account>::as_ref(failure).to_string()).collect();
    assert_eq!(told, ["went on, then halted", "stalled"]);
    Ok(())
}

#[test]
fn a_held_segment_is_held_under_the_job_its_run_belongs_to() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(store.read(Holding::from(Recorded::session()?.records))?)?;
    let [first, second] = [JobId::new(1), JobId::new(2)];
    store.write(JobOpened::from(first))?;
    store.write(JobOpened::from(second))?;
    let spawned = Spawned::new(Pid::new(7), None, Deadline::new(Wall::new(0)));
    store.write(RunSpawned::new(RunId::new(1), first, spawned))?;
    let segments = vec![Proposals::segment("o1", 0)?];
    let flail = Some(JobFlailed::new(RunId::new(1), segments));
    let misfiled =
        store.write(JobHalted::new(second, Halted::Flailed, Account::told(&"flailed")).holding(flail.clone()));
    assert!(misfiled.is_err());
    assert_eq!(store.read(HeldBy::from(second))?, None);
    store.write(JobHalted::new(first, Halted::Flailed, Account::told(&"flailed")).holding(flail))?;
    let hold = store.read(HeldBy::from(first))?.ok_or("no hold")?;
    let filed: Vec<(JobId, RunId)> = hold.held().map(|flailed| (flailed.job(), flailed.run())).collect();
    assert_eq!(filed, vec![(first, RunId::new(1))]);
    Ok(())
}

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;
