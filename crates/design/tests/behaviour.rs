use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use std::sync::Mutex;

use console_ports::Variables;
use design::{Cli, Failure, Outcome, Unresolved};
use design_model::{Entered, Entry, Facet, Node, NodeEdge, NodeId, NodeRelation, Role, SegmentRelation, Stage};
use design_store::{AppliedId, Linked, Planting, ShownMessage, ShownNode, Stored, StreamSchema};
use file_ports::{Bytes, FileFailure, Files, Kind, Memory, Size, Text};
use laws::proptest::prelude::{prop_assert, prop_assert_eq};
use laws::proptest::sample::select;
use laws::proptest::test_runner::TestCaseError;
use newtype::Counter;
use port_core::{Held, Pass, Release, ReleaseReceipt, Released, Resource, Run, port};
use store_ports::{Busy, Sqlite, Store, Written};
use strum::VariantArray;
use transcript_ports::Uuid;

const HOME: &str = "/home/tester";

#[derive(Clone, Copy)]
enum Speaker {
    Owner,
    Lead,
    Unheard,
}

struct Wire {
    kind: &'static str,
    origin: &'static str,
}

impl Speaker {
    fn wire(self) -> Wire {
        match self {
            Speaker::Owner => Wire { kind: "user", origin: "{\"kind\":\"human\"}" },
            Speaker::Lead => Wire { kind: "assistant", origin: "null" },
            Speaker::Unheard => Wire { kind: "brand-new-kind", origin: "null" },
        }
    }

    fn content(self, text: &str) -> String {
        match self {
            Speaker::Lead => format!("[{{\"type\":\"text\",\"text\":\"{text}\"}}]"),
            Speaker::Owner | Speaker::Unheard => format!("\"{text}\""),
        }
    }
}

struct Session {
    directory: &'static str,
    lines: String,
}

impl Session {
    fn working(lines: String) -> Session {
        Session { directory: "/home/tester/.claude/projects/-work", lines }
    }

    fn line(speaker: Speaker, uuid: &str, parent: &str, text: &str) -> String {
        let Wire { kind, origin } = speaker.wire();
        let content = speaker.content(text);
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":{parent},\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"model\":\"claude\",\"content\":{content}}}}}\n"
        )
    }

    fn answered() -> Session {
        Session::working(
            [
                Session::line(Speaker::Owner, "o1", "null", "question"),
                Session::line(Speaker::Lead, "l1", "\"o1\"", "answer"),
            ]
            .concat(),
        )
    }

    fn typical() -> Session {
        Session::working(
            [
                Self::line(Speaker::Owner, "o1", "null", "1. go. now"),
                Self::line(Speaker::Lead, "l1", "null", "done"),
                Self::line(Speaker::Owner, "o2", "null", "thanks"),
                Self::line(Speaker::Unheard, "b1", "null", "result"),
            ]
            .concat(),
        )
    }

    fn linked() -> Session {
        Session::working(
            [
                Self::line(Speaker::Owner, Self::ASKED, "null", "what of aaaaaaaa-0000-4000-8000-000000000004"),
                Self::line(
                    Speaker::Lead,
                    "aaaaaaaa-0000-4000-8000-000000000002",
                    "\"aaaaaaaa-0000-4000-8000-000000000001\"",
                    "1. a\\n2. b",
                ),
                Self::line(Speaker::Unheard, "b1", "\"aaaaaaaa-0000-4000-8000-000000000002\"", "result"),
                Self::line(Speaker::Owner, "aaaaaaaa-0000-4000-8000-000000000003", "\"b1\"", "1. yes\\n2. no"),
                Self::line(
                    Speaker::Lead,
                    "aaaaaaaa-0000-4000-8000-000000000004",
                    "\"aaaaaaaa-0000-4000-8000-000000000003\"",
                    "see aaaaaaaa-0000-4000-8000-000000000001",
                ),
                Self::line(
                    Speaker::Owner,
                    "aaaaaaaa-0000-4000-8000-000000000005",
                    "\"aaaaaaaa-0000-4000-8000-000000000004\"",
                    "what of aaaaaaaa-0000-4000-8000-000000000004",
                ),
            ]
            .concat(),
        )
    }

    const ASKED: &str = "aaaaaaaa-0000-4000-8000-000000000001";

    const SAID: [&str; 5] = [
        Self::ASKED,
        "aaaaaaaa-0000-4000-8000-000000000002",
        "aaaaaaaa-0000-4000-8000-000000000003",
        "aaaaaaaa-0000-4000-8000-000000000004",
        "aaaaaaaa-0000-4000-8000-000000000005",
    ];

    fn prefix(&self, cut: usize) -> Session {
        Session { directory: self.directory, lines: self.lines.chars().take(cut).collect() }
    }

    fn files(&self) -> Memory {
        let memory = Memory::default();
        memory.create_directories(Path::new(self.directory)).ok();
        memory.write(&Path::new(self.directory).join("s.jsonl"), Bytes::from(&self.lines)).ok();
        memory
    }
}

#[derive(Debug)]
struct FakeLent<'memory> {
    memory: &'memory Memory,
}

impl FakeLent<'_> {
    fn new(memory: &Memory) -> FakeLent<'_> {
        FakeLent { memory }
    }
}

impl Files for FakeLent<'_> {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.memory.read(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        self.memory.read_text(path)
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.memory.write(path, bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        self.memory.append(path, bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        self.memory.create_directories(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.memory.copy(from, to)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        self.memory.entries(directory)
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        self.memory.kind(path)
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.memory.size(path)
    }
}

struct Invocation;

impl Invocation {
    fn environment() -> Variables {
        Invocation::at("/work")
    }

    fn at(directory: &str) -> Variables {
        [("HOME", HOME), ("PWD", directory)]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect()
    }

    fn output(session: &Session, arguments: &[&str]) -> Result<String, Failure> {
        Self::against(&Sqlite::in_memory::<StreamSchema>(), session, arguments)
    }

    fn against(store: &Sqlite, session: &Session, arguments: &[&str]) -> Result<String, Failure> {
        Self::within(store, session.files(), arguments)
    }

    fn within(store: &Sqlite, files: impl Files, arguments: &[&str]) -> Result<String, Failure> {
        let run = Run::root();
        let mut out = Vec::new();
        Cli::new(&Self::environment(), files, |_path| store.clone()).report(
            &run,
            arguments.iter().map(OsString::from),
            &mut out,
        )?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }

    fn shown(session: &Session, uuid: &str) -> Result<String, Failure> {
        let store = Sqlite::in_memory::<StreamSchema>();
        Self::against(&store, session, &["ingest"])?;
        Self::against(&store, session, &["show", uuid])
    }
}

#[test]
fn ingest_prints_no_unrecognised_line_when_every_record_is_recognised() -> Result<(), Failure> {
    let out = Invocation::output(&Session::answered(), &["ingest"])?;
    assert_eq!(out.matches("unrecognised").count(), 0);
    Ok(())
}

#[test]
fn a_relative_workspace_is_resolved_against_the_working_directory() -> Result<(), Failure> {
    let out = Invocation::output(&Session::answered(), &["ingest", "--workspace", "."])?;
    assert!(out.contains("authorship owner 1\n"));
    Ok(())
}

#[test]
fn ingest_counts_each_authorship_unrecognised_kind_form_and_relation() -> Result<(), Failure> {
    let out = Invocation::output(&Session::typical(), &["ingest"])?;
    assert!(out.contains("authorship owner 2\n"));
    assert!(out.contains("authorship lead 1\n"));
    assert!(out.contains("unrecognised brand-new-kind 1\n"));
    assert!(out.contains("form item 1\n"));
    assert!(out.contains("form sentence 2\n"));
    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Link {
    source: String,
    relation: String,
    target: String,
}

struct Links;

impl Links {
    fn stored(store: &Sqlite) -> Result<BTreeSet<Link>, Failure> {
        let run = Run::root();
        let port = run.port(store.clone());
        let shown = Session::SAID
            .iter()
            .map(|uuid| {
                let uuid = Uuid::new((*uuid).to_owned());
                port.read(ShownMessage::named(uuid.clone()))?.ok_or(Failure::Missing(uuid))
            })
            .collect::<Result<Vec<_>, Failure>>()?;
        Ok(shown
            .iter()
            .flat_map(Stored::links)
            .map(|link| {
                let [source, target] = link.anchors();
                Link { source: source.to_string(), relation: link.relation().to_string(), target: target.to_string() }
            })
            .collect())
    }

    fn whole(session: &Session) -> Result<BTreeSet<Link>, Failure> {
        let store = Sqlite::in_memory::<StreamSchema>();
        Invocation::against(&store, session, &["ingest"])?;
        Self::stored(&store)
    }

    fn split(session: &Session, cut: usize) -> Result<BTreeSet<Link>, Failure> {
        let store = Sqlite::in_memory::<StreamSchema>();
        Invocation::against(&store, &session.prefix(cut), &["ingest"])?;
        Invocation::against(&store, session, &["ingest"])?;
        Self::stored(&store)
    }
}

struct Answered {
    outcome: Outcome,
    out: String,
}

struct Rooted;

impl Rooted {
    fn outcome(store: &Sqlite, session: &Session, arguments: &[&str]) -> Result<Answered, Failure> {
        let mut out = Vec::new();
        let outcome = Cli::new(&Invocation::environment(), session.files(), |_path| store.clone())
            .rooted(arguments.iter().map(OsString::from), &mut out)?;
        Ok(Answered { outcome, out: String::from_utf8(out).unwrap_or_default() })
    }
}

#[test]
fn a_rooted_run_writes_its_report_to_out_and_names_its_outcome() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let ingested = Rooted::outcome(&store, &Session::typical(), &["ingest"])?;
    assert_eq!(ingested.outcome, Outcome::Ingested);
    assert!(ingested.out.contains("authorship owner 2\n"));
    let shown = Rooted::outcome(&store, &Session::typical(), &["show", "o1"])?;
    assert_eq!(shown.outcome, Outcome::Shown);
    Ok(())
}

#[test]
fn show_lists_the_links_that_touch_the_message() -> Result<(), Failure> {
    let out = Invocation::shown(&Session::answered(), "o1")?;
    assert!(out.contains("link l1 replies-to o1"));
    let unrelated = Invocation::shown(&Session::typical(), "o2")?;
    assert_eq!(unrelated.matches("link").count(), 0);
    Ok(())
}

#[test]
fn ingest_counts_a_reply_once() -> Result<(), Failure> {
    let out = Invocation::output(&Session::answered(), &["ingest"])?;
    assert!(out.contains("relation replies-to 1\n"));
    Ok(())
}

#[test]
fn show_prints_a_message_by_uuid_from_the_store() -> Result<(), Failure> {
    let out = Invocation::shown(&Session::typical(), "o1")?;
    assert!(out.contains("o1 owner session s at 2026-01-01T00:00:00Z"));
    assert!(out.contains("0 item"));
    Ok(())
}

#[test]
fn show_of_an_unknown_uuid_fails_naming_it() {
    let failed = Invocation::shown(&Session::typical(), "nobody");
    assert!(matches!(failed, Err(Failure::Missing(_))));
}

#[test]
fn the_workspace_flag_picks_the_projects_directory() -> Result<(), Failure> {
    let session = Session { directory: "/home/tester/.claude/projects/-other", lines: Session::typical().lines };
    let after = Invocation::output(&session, &["ingest", "--workspace", "/other"])?;
    let before = Invocation::output(&session, &["--workspace", "/other", "ingest"])?;
    assert!(after.contains("authorship owner 2\n"));
    assert_eq!(before, after);
    Ok(())
}

#[test]
fn an_unknown_command_is_a_usage_failure() {
    assert!(matches!(Invocation::output(&Session::typical(), &["frobnicate"]), Err(Failure::Usage)));
}

#[test]
fn a_workspace_flag_without_a_path_is_a_usage_failure() {
    assert!(matches!(Invocation::output(&Session::typical(), &["ingest", "--workspace"]), Err(Failure::Usage)));
}

#[test]
fn show_reads_the_store_not_the_live_transcript() {
    let store = Sqlite::in_memory::<StreamSchema>();
    let before = Invocation::against(&store, &Session::typical(), &["show", "o1"]);
    assert!(matches!(before, Err(Failure::Missing(_))));
}

struct Opened {
    paths: Vec<PathBuf>,
    out: String,
}

impl Opened {
    fn by(arguments: &[&str]) -> Result<Opened, Failure> {
        Opened::from("/work", Session::typical().files(), arguments)
    }

    fn from(directory: &str, files: impl Files, arguments: &[&str]) -> Result<Opened, Failure> {
        let opened = Mutex::new(Vec::new());
        let run = Run::root();
        let mut out = Vec::new();
        Cli::new(&Invocation::at(directory), files, |path: PathBuf| {
            opened.lock().map(|mut paths| paths.push(path)).ok();
            Sqlite::in_memory::<StreamSchema>()
        })
        .report(&run, arguments.iter().map(OsString::from), &mut out)?;
        Ok(Opened { paths: opened.into_inner().unwrap_or_default(), out: String::from_utf8(out).unwrap_or_default() })
    }
}

#[test]
fn the_store_lives_in_the_workspace_meta_directory() -> Result<(), Failure> {
    assert_eq!(Opened::by(&["ingest"])?.paths, [PathBuf::from("/work/.meta/design.sqlite")]);
    Ok(())
}

#[test]
fn a_dry_run_ingest_prints_the_counts_and_leaves_the_workspace_store_absent() -> Result<(), Failure> {
    let dry = Opened::by(&["ingest", "--dry-run"])?;
    assert!(dry.out.contains("authorship owner 2\n"));
    assert_eq!(dry.paths, Vec::<PathBuf>::new());
    Ok(())
}

#[test]
fn a_second_ingest_over_unchanged_transcripts_writes_nothing_new() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let first = Invocation::against(&store, &Session::answered(), &["ingest"])?;
    let second = Invocation::against(&store, &Session::answered(), &["ingest"])?;
    assert!(first.contains("written 8\n"));
    assert!(second.contains("written 0\n"));
    assert_eq!(second.matches("authorship").count(), 0);
    Ok(())
}

#[test]
fn ingest_resumes_each_session_from_its_stored_cursor() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let first = Session::working(Session::line(Speaker::Owner, "o1", "null", "question"));
    Invocation::against(&store, &first, &["ingest"])?;
    let resumed = Invocation::against(&store, &Session::answered(), &["ingest"])?;
    assert!(resumed.contains("authorship lead 1\n"));
    assert_eq!(resumed.matches("authorship owner").count(), 0);
    assert!(Invocation::against(&store, &Session::answered(), &["show", "l1"])?.contains("l1 lead session s"));
    Ok(())
}

#[test]
fn a_second_ingest_is_refused_while_the_store_lock_is_held() -> Result<(), Busy> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let holder = Run::root();
    let port = holder.port(store.clone());
    let held = port.lock()?;
    let refused = Invocation::against(&store, &Session::answered(), &["ingest"]);
    assert!(matches!(refused, Err(Failure::Busy(_))));
    drop(held);
    assert!(Invocation::against(&store, &Session::answered(), &["ingest"]).is_ok());
    Ok(())
}

#[test]
fn a_reply_appended_after_an_ingest_links_to_the_message_stored_before_it() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let asked = Session::line(Speaker::Owner, "o1", "null", "question");
    let answered = Session::line(Speaker::Lead, "l1", "\"o1\"", "answer");
    let thanked = Session::line(Speaker::Owner, "o2", "\"l1\"", "thanks");
    Invocation::against(&store, &Session::working(asked.clone()), &["ingest"])?;
    Invocation::against(&store, &Session::working([asked.clone(), answered.clone()].concat()), &["ingest"])?;
    let shown = Invocation::against(&store, &Session::answered(), &["show", "o1"])?;
    assert!(shown.contains("link l1 replies-to o1"));
    let resumed = Invocation::against(&store, &Session::working([asked, answered, thanked].concat()), &["ingest"])?;
    assert!(resumed.contains("relation replies-to 1\n"));
    Ok(())
}

laws::law!(ingesting_a_transcript_in_two_parts_stores_the_links_of_ingesting_it_whole, 0usize..1400, cut, {
    let session = Session::linked();
    let whole = Links::whole(&session).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let split = Links::split(&session, cut).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let relations: BTreeSet<&str> = whole.iter().map(|link| link.relation.as_str()).collect();
    prop_assert_eq!(relations, BTreeSet::from(["answers", "cites", "replies-to", "resends"]));
    prop_assert_eq!(split, whole);
});

struct Found;

impl Found {
    fn by(session: &Session, arguments: &[&str]) -> Result<String, TestCaseError> {
        let store = Sqlite::in_memory::<StreamSchema>();
        Invocation::against(&store, session, &["ingest"])
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        Invocation::against(&store, session, arguments).map_err(|failure| TestCaseError::fail(failure.to_string()))
    }

    fn said(lines: &[Said]) -> Session {
        Session::working(lines.iter().map(|said| Session::line(said.speaker, said.uuid, "null", &said.text)).collect())
    }
}

struct Said {
    speaker: Speaker,
    uuid: &'static str,
    text: String,
}

impl Said {
    fn by(speaker: Speaker, uuid: &'static str, text: String) -> Said {
        Said { speaker, uuid, text }
    }
}

laws::law!(an_owner_segment_is_found_by_a_word_it_contains_and_no_other_is, "[a-z]{4,10}", word, {
    let session = Found::said(&[
        Said::by(Speaker::Owner, "o1", format!("alpha {word}")),
        Said::by(Speaker::Owner, "o2", "1234 5678".to_owned()),
    ]);
    let out = Found::by(&session, &["search", &word])?;
    prop_assert!(out.contains("o1 owner"));
    prop_assert!(!out.contains("o2 owner"));
});

laws::law!(several_words_find_only_the_segments_holding_every_one, ("a[a-z]{3,8}", "b[a-z]{3,8}"), words, {
    let (first, second) = words;
    let session = Found::said(&[
        Said::by(Speaker::Owner, "o1", format!("{second} and {first}")),
        Said::by(Speaker::Owner, "o2", format!("{first} 1234")),
        Said::by(Speaker::Owner, "o3", format!("5678 {second}")),
    ]);
    let out = Found::by(&session, &["search", &first, &second])?;
    prop_assert_eq!(out.matches(" owner ").count(), 1);
    prop_assert!(out.contains("o1 owner"));
});

laws::law!(the_lead_is_searched_only_when_everyone_is_asked_for, "[a-z]{4,10}", word, {
    let session =
        Found::said(&[Said::by(Speaker::Owner, "o1", word.clone()), Said::by(Speaker::Lead, "l1", word.clone())]);
    let owner = Found::by(&session, &["search", &word])?;
    let everyone = Found::by(&session, &["search", "--all", &word])?;
    prop_assert!(owner.contains("o1 owner"));
    prop_assert!(!owner.contains("l1 lead"));
    prop_assert!(everyone.contains("o1 owner"));
    prop_assert!(everyone.contains("l1 lead"));
});

laws::law!(a_word_is_found_whatever_its_case, "[a-z]{4,10}", word, {
    let raised: String = word.chars().map(|letter| letter.to_ascii_uppercase()).collect();
    let session =
        Found::said(&[Said::by(Speaker::Owner, "o1", raised.clone()), Said::by(Speaker::Owner, "o2", word.clone())]);
    let lower = Found::by(&session, &["search", &word])?;
    let upper = Found::by(&session, &["search", &raised])?;
    prop_assert!(lower.contains("o1 owner") && lower.contains("o2 owner"));
    prop_assert!(upper.contains("o1 owner") && upper.contains("o2 owner"));
});

#[test]
fn a_search_without_words_is_a_usage_failure() {
    assert!(matches!(Invocation::output(&Session::typical(), &["search", "--all"]), Err(Failure::Usage)));
    assert!(matches!(Invocation::output(&Session::typical(), &["search", "..."]), Err(Failure::Usage)));
}

#[test]
fn a_rooted_search_names_its_outcome() -> Result<(), Failure> {
    let searched = Rooted::outcome(&Sqlite::in_memory::<StreamSchema>(), &Session::typical(), &["search", "thanks"])?;
    assert_eq!(searched.outcome, Outcome::Searched);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("the lock was already gone")]
struct Gone;

#[derive(Debug, Resource)]
struct Broken;

impl Release for Broken {
    type Failure = Gone;

    fn release(&mut self, _pass: Pass<'_, Released<Broken>>) -> Result<ReleaseReceipt, Gone> {
        Err(Gone)
    }
}

#[port]
trait Locks {
    fn lock(&self) -> Held<'_, Broken>;
}

struct Locker;

impl LocksFiller for Locker {
    fn lock(&self, _pass: Pass<'_, locks_ops::Lock>) -> Broken {
        Broken
    }
}

#[test]
fn a_failed_command_keeps_the_release_failures_of_its_run() {
    let run = Run::root();
    drop(run.port(Locker).lock());
    let settled = Failure::settled(Err(Failure::Usage), run.finish().settled());
    assert!(matches!(settled, Err(Failure::Unreleased { command, .. }) if matches!(*command, Failure::Usage)));
}

struct Graph;

impl Graph {
    fn init(store: &Sqlite, arguments: &[&str]) -> Result<String, Failure> {
        Invocation::against(store, &Session::typical(), &[&["init"], arguments].concat())
    }

    fn shown(store: &Sqlite, node: &str) -> Result<String, Failure> {
        Invocation::against(store, &Session::typical(), &["show", node])
    }
}

#[test]
fn init_plants_the_project_root_in_the_workspace_store() -> Result<(), Failure> {
    let opened = Opened::by(&["init", "demo"])?;
    assert_eq!(opened.paths, [PathBuf::from("/work/.meta/design.sqlite")]);
    assert!(opened.out.contains("root n1 project demo"));
    Ok(())
}

#[test]
fn a_rooted_init_names_its_outcome() -> Result<(), Failure> {
    let initialised = Rooted::outcome(&Sqlite::in_memory::<StreamSchema>(), &Session::typical(), &["init", "demo"])?;
    assert_eq!(initialised.outcome, Outcome::Initialised);
    Ok(())
}

#[test]
fn a_second_init_changes_nothing() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    let first = Graph::init(&store, &["demo"])?;
    let before = Graph::shown(&store, "n1")?;
    let second = Graph::init(&store, &["demo"])?;
    assert_eq!(second, first);
    assert_eq!(Graph::shown(&store, "n1")?, before);
    assert!(matches!(Graph::shown(&store, "n2"), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn init_links_a_consumed_project_only_when_asked() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    Graph::init(&store, &["meta-dev"])?;
    Graph::init(&store, &["demo"])?;
    assert_eq!(Graph::shown(&store, "n2")?.matches("consumes").count(), 0);
    let consuming = Graph::init(&store, &["demo", "--consumes", "meta-dev"])?;
    assert!(consuming.contains("consumes n1 project meta-dev"));
    assert!(
        Graph::shown(&store, "n2")?.contains("n2 demo project vague core entered init\n  consumes n1 entered init\n")
    );
    assert_eq!(Graph::shown(&store, "n1")?.matches("consumes").count(), 0);
    Ok(())
}

#[test]
fn init_consuming_a_project_without_a_root_fails_and_plants_nothing() {
    let store = Sqlite::in_memory::<StreamSchema>();
    assert!(matches!(Graph::init(&store, &["demo", "--consumes", "nowhere"]), Err(Failure::Unrooted(_))));
    assert!(matches!(Graph::shown(&store, "n1"), Err(Failure::Unplaced(_))));
}

#[test]
fn init_refuses_a_project_consuming_itself_and_plants_nothing() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    Graph::init(&store, &["demo"])?;
    assert!(matches!(Graph::init(&store, &["demo", "--consumes", "demo"]), Err(Failure::SelfConsuming(_))));
    assert_eq!(Graph::shown(&store, "n1")?.matches("consumes").count(), 0);
    Ok(())
}

#[test]
fn a_misnumbered_node_id_is_refused_as_a_node_id_not_looked_up_as_a_message() {
    let store = Sqlite::in_memory::<StreamSchema>();
    assert!(matches!(Graph::shown(&store, "n001"), Err(Failure::Unnumbered(_))));
    assert!(matches!(Graph::shown(&store, "n99999999999999999999"), Err(Failure::Unnumbered(_))));
}

#[test]
fn only_ingest_has_a_dry_run() {
    let store = Sqlite::in_memory::<StreamSchema>();
    let refused = [
        Graph::init(&store, &["demo", "--dry-run"]),
        Graph::shown(&store, "--dry-run"),
        Invocation::against(&store, &Session::typical(), &["show", "o1", "--dry-run"]),
        Invocation::against(&store, &Session::typical(), &["search", "thanks", "--dry-run"]),
        Invocation::against(&store, &Session::typical(), &["--dry-run", "ingest"]),
        Invocation::against(&store, &Session::typical(), &["ingest", "--dry-run", "--dry-run"]),
    ];
    assert!(refused.iter().all(|refused| matches!(refused, Err(Failure::Usage))));
}

#[test]
fn init_without_a_project_or_a_consumed_name_is_a_usage_failure() {
    let store = Sqlite::in_memory::<StreamSchema>();
    assert!(matches!(Graph::init(&store, &[]), Err(Failure::Usage)));
    assert!(matches!(Graph::init(&store, &["demo", "--consumes"]), Err(Failure::Usage)));
}

#[test]
fn show_of_an_unknown_node_fails_naming_it() {
    let store = Sqlite::in_memory::<StreamSchema>();
    assert!(matches!(Graph::shown(&store, "n7"), Err(Failure::Unplaced(_))));
}

struct Repository {
    files: Memory,
}

impl Repository {
    fn at(root: &str) -> Repository {
        let files = Session::typical().files();
        files.create_directories(&Path::new(root).join(".git")).ok();
        Repository { files }
    }

    fn opened(&self, directory: &str, arguments: &[&str]) -> Result<Vec<PathBuf>, TestCaseError> {
        Opened::from(directory, FakeLent::new(&self.files), arguments)
            .map(|opened| opened.paths)
            .map_err(|failure| TestCaseError::fail(failure.to_string()))
    }
}

laws::law!(
    init_from_a_subdirectory_records_the_workspace_at_the_repository_root,
    "[a-z]{1,6}(/[a-z]{1,6}){0,2}",
    below,
    {
        let repository = Repository::at("/work");
        let inside = format!("/work/{below}");
        repository.opened(&inside, &["init", "demo", "--workspace", "/ws"])?;
        prop_assert_eq!(repository.files.kind(Path::new("/work/.meta/local.toml")), Kind::File);
        prop_assert_eq!(repository.files.kind(&Path::new(&inside).join(".meta")), Kind::Missing);
        prop_assert_eq!(repository.opened(&inside, &["search", "thanks"])?, [PathBuf::from("/ws/.meta/design.sqlite")]);
        prop_assert_eq!(repository.opened("/work", &["search", "thanks"])?, [PathBuf::from("/ws/.meta/design.sqlite")]);
    }
);

laws::law!(the_workspace_flag_overrides_the_recorded_workspace, "/[a-z]{1,6}", flagged, {
    let repository = Repository::at("/work");
    repository.opened("/work", &["init", "demo", "--workspace", "/ws"])?;
    let opened = repository.opened("/work", &["search", "thanks", "--workspace", &flagged])?;
    prop_assert_eq!(opened, [Path::new(&flagged).join(".meta/design.sqlite")]);
});

#[test]
fn without_a_repository_or_a_recorded_workspace_the_working_directory_is_the_workspace() -> Result<(), Failure> {
    let opened = Opened::from("/work/sub", Session::typical().files(), &["search", "thanks"])?;
    assert_eq!(opened.paths, [PathBuf::from("/work/sub/.meta/design.sqlite")]);
    Ok(())
}

struct Lines;

impl Lines {
    fn missing<'wanted, Line: AsRef<str>>(out: &str, wanted: &'wanted [Line]) -> Vec<&'wanted Line> {
        wanted.iter().filter(|line| out.matches(&format!("{}\n", line.as_ref())).count() == 0).collect()
    }
}

struct Applying {
    store: Sqlite,
    files: Memory,
}

impl Applying {
    const ANSWER: &str = "aaaaaaaa-0000-4000-8000-000000000003";

    fn ready() -> Result<Applying, Failure> {
        let applying = Applying { store: Sqlite::in_memory::<StreamSchema>(), files: Session::linked().files() };
        applying.run(&["ingest"])?;
        applying.run(&["init", "demo"])?;
        Ok(applying)
    }

    fn run(&self, arguments: &[&str]) -> Result<String, Failure> {
        Invocation::within(&self.store, FakeLent::new(&self.files), arguments)
    }

    fn applied(&self, directives: &str) -> Result<String, Failure> {
        self.files.create_directories(Path::new("/work/notes")).ok();
        self.files.write(Path::new("/work/notes/directives"), Bytes::from(directives)).ok();
        self.run(&["apply", "notes/directives"])
    }

    fn answer(ordinal: usize) -> String {
        format!("{}.{ordinal}", Self::ANSWER)
    }
}

#[test]
fn apply_places_proposes_and_reports_the_owner_segments_it_leaves_uncovered() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let answer = Applying::answer(0);
    let directives = [
        format!("place({answer}, n1, decides)"),
        "propose(new, refines(n1), feature, \"Apply \\\"directives\\\"\")".to_owned(),
        format!("sense({answer}, decision, specific, firm, explicit)"),
        format!("link({answer}, aaaaaaaa-0000-4000-8000-000000000002.0, answers)"),
    ]
    .join("\n");
    let out = applying.applied(&directives)?;
    let wanted = [
        format!("applied place({answer}, n1, decides)"),
        "proposed n2 by propose(new, refines(n1), feature, \"Apply \\\"directives\\\"\")".to_owned(),
        format!("applied sense({answer}, decision, specific, firm, explicit)"),
        format!("applied link({answer}, aaaaaaaa-0000-4000-8000-000000000002.0, answers)"),
        format!("uncovered {}", Applying::answer(1)),
        "written 7".to_owned(),
    ];
    assert_eq!(Lines::missing(&out, &wanted), Vec::<&String>::new());
    assert_eq!(out.matches('\n').count(), wanted.len());
    Ok(())
}

#[test]
fn a_proposal_is_a_vague_child_shown_with_its_ancestors_and_children() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.applied("propose(new, refines(n1), feature, \"apply\")\npropose(new, refines(n1), smell, \"drift\")")?;
    applying.applied("propose(new, refines(n2), feature, \"echo\")")?;
    let child = applying.run(&["show", "n4"])?;
    let wanted = [
        "n4 demo feature vague core entered apply \"echo\"",
        "  ancestor n1 demo project vague core entered init",
        "  ancestor n2 demo feature vague core entered apply \"apply\"",
        "  refines n2 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&child, &wanted), Vec::<&&str>::new());
    assert_eq!(child.matches('\n').count(), wanted.len());
    let root = applying.run(&["show", "n1"])?;
    assert_eq!(root.matches("  child ").count(), 2);
    assert!(root.contains("  child n3 demo smell vague core entered apply \"drift\"\n"));
    Ok(())
}

#[test]
fn a_superseded_node_stays_resolvable() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.applied("propose(new, refines(n1), feature, \"old\")\npropose(new, refines(n1), feature, \"new\")")?;
    applying.applied("supersede(n2, by(n3))")?;
    let root = "  ancestor n1 demo project vague core entered init";
    let refines = "  refines n1 entered apply";
    let cast = "  role core entered apply";
    let wanted = [
        ("n2", vec!["n2 demo feature vague core entered apply \"old\"", root, refines, cast]),
        (
            "n3",
            vec![
                "n3 demo feature vague core entered apply \"new\"",
                root,
                refines,
                "  supersedes n2 entered apply",
                cast,
            ],
        ),
    ];
    wanted.iter().try_for_each(|(node, lines)| {
        let shown = applying.run(&["show", node])?;
        assert_eq!(Lines::missing(&shown, lines), Vec::<&&str>::new());
        assert_eq!(shown.matches('\n').count(), lines.len());
        Ok::<(), Failure>(())
    })?;
    let placed = applying.applied(&format!("place({}, n2, mentions)", Applying::answer(1)))?;
    assert!(placed.contains("written 2\n"));
    Ok(())
}

impl Applying {
    fn refused(&self, directives: &str) -> Option<Failure> {
        self.applied(&format!("propose(new, refines(n1), feature, \"first\")\n{directives}")).err()
    }
}

#[test]
fn an_unknown_reference_is_refused_and_nothing_is_written() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let refusals = [
        "conversational(ffffffff.0)".to_owned(),
        "conversational(aaaaaaaa.0)".to_owned(),
        format!("conversational({})", Applying::answer(7)),
        format!("place({}, n9, defines)", Applying::answer(0)),
        "supersede(n1, by(n9))".to_owned(),
        "place(n1, n1, defines)".to_owned(),
    ]
    .map(|directives| applying.refused(&directives));
    assert!(matches!(
        refusals,
        [
            Some(Failure::Unresolved(Unresolved::Unheard(_))),
            Some(Failure::Unresolved(Unresolved::Ambiguous(_))),
            Some(Failure::Unresolved(Unresolved::Unsegmented(_))),
            Some(Failure::Unresolved(Unresolved::Unplaced(_))),
            Some(Failure::Unresolved(Unresolved::Unplaced(_))),
            Some(Failure::Misread(_)),
        ]
    ));
    assert!(matches!(applying.run(&["show", "n2"]), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn a_supersession_across_project_roots_is_refused_naming_both_nodes_and_nothing_is_written() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let run = Run::root();
    let entry = Entry::new("other".parse()?, Facet::Project, Entered::Init.into());
    run.port(applying.store.clone()).write(Planting::new(Some(Node::new(NodeId::new(2), entry)), None))?;
    assert!(matches!(applying.refused("supersede(n1, by(n2))"), Some(Failure::Unresolved(Unresolved::Straddling(_)))));
    assert!(matches!(applying.run(&["show", "n3"]), Err(Failure::Unplaced(_))));
    assert_eq!(applying.run(&["show", "n2"])?.matches("supersedes").count(), 0);
    Ok(())
}

impl Applying {
    fn cyclic(&self, directives: &str) -> Option<Failure> {
        self.applied("propose(new, refines(n1), feature, \"a\")\npropose(new, refines(n1), feature, \"b\")").ok();
        self.applied(directives).err()
    }
}

#[test]
fn a_supersession_that_closes_a_cycle_or_names_one_node_twice_is_refused() -> Result<(), Failure> {
    let refusals = [
        "supersede(n2, by(n2))",
        "supersede(n2, by(n3))\nsupersede(n3, by(n2))",
        "supersede(n2, by(n3))\nsupersede(n3, by(n1))\nsupersede(n1, by(n2))",
        "propose(new x, refines(n1), feature, \"c\")\nsupersede(n4, by(x))",
    ]
    .map(|directives| Applying::ready().ok().and_then(|applying| applying.cyclic(directives)));
    assert!(matches!(
        refusals,
        [
            Some(Failure::Misread(_)),
            Some(Failure::Unresolved(Unresolved::Cyclic { .. })),
            Some(Failure::Unresolved(Unresolved::Cyclic { .. })),
            Some(Failure::Unresolved(Unresolved::Cyclic { .. })),
        ]
    ));
    let applying = Applying::ready()?;
    applying.cyclic("supersede(n2, by(n3))");
    assert!(matches!(applying.cyclic("supersede(n3, by(n2))"), Some(Failure::Unresolved(Unresolved::Cyclic { .. }))));
    assert!(applying.applied("propose(new, refines(n1), feature, \"d\")\nsupersede(n3, by(n4))").is_ok());
    Ok(())
}

#[test]
fn a_segment_covered_by_an_earlier_apply_is_not_reported_uncovered() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let first = applying.applied(&format!("conversational({})", Applying::answer(0)))?;
    assert!(first.contains(&format!("uncovered {}\n", Applying::answer(1))));
    let second = applying.applied(&format!("sense({}, decision, specific, firm, explicit)", Applying::answer(1)))?;
    assert_eq!(second.matches("uncovered").count(), 0);
    Ok(())
}

impl Applying {
    fn linked(&self, relation: &str, ends: &[[usize; 2]]) -> Result<String, Failure> {
        let lines: Vec<String> = ends
            .iter()
            .map(|[source, target]| {
                format!("link({}, {}, {relation})", Applying::answer(*source), Applying::answer(*target))
            })
            .collect();
        self.applied(&lines.join("\n"))
    }
}

#[test]
fn a_segment_supersession_or_correction_that_closes_a_cycle_is_refused() {
    let refusals = [["supersedes", "corrects"], ["corrects", "corrects"]].map(|[first, second]| {
        Applying::ready().ok().and_then(|applying| {
            applying.linked(first, &[[0, 1]]).ok();
            applying.linked(second, &[[1, 0]]).err()
        })
    });
    assert!(matches!(refusals, [None, Some(Failure::Unresolved(Unresolved::Circular { .. }))]));
    let ordered = ["supersedes", "corrects"]
        .map(|relation| Applying::ready().ok().and_then(|applying| applying.linked(relation, &[[0, 1], [1, 0]]).err()));
    assert!(matches!(
        ordered,
        [
            Some(Failure::Unresolved(Unresolved::Circular { .. })),
            Some(Failure::Unresolved(Unresolved::Circular { .. }))
        ]
    ));
    let mutual = ["answers", "refers-to"]
        .map(|relation| Applying::ready().ok().map(|applying| applying.linked(relation, &[[0, 1], [1, 0]]).is_ok()));
    assert_eq!(mutual, [Some(true), Some(true)]);
}

#[test]
fn a_segment_linked_to_itself_under_another_spelling_is_refused_for_every_relation() {
    let refusals: Vec<Option<Failure>> = SegmentRelation::VARIANTS
        .iter()
        .map(|relation| {
            let said = Session::line(Speaker::Owner, "cccccccc-0000-4000-8000-000000000001", "null", "alone");
            let applying =
                Applying { store: Sqlite::in_memory::<StreamSchema>(), files: Session::working(said).files() };
            applying.run(&["ingest"]).ok();
            applying.applied(&format!("link(cccccccc.0, cccccccc-0000.0, {relation})")).err()
        })
        .collect();
    assert_eq!(refusals.len(), SegmentRelation::VARIANTS.len());
    assert!(refusals.iter().all(|refused| matches!(refused, Some(Failure::Unresolved(Unresolved::Looped(_))))));
}

#[test]
fn applied_annotations_show_their_provenance() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let answer = Applying::answer(0);
    let directives = [
        "propose(new x, refines(n1), feature, \"a\")".to_owned(),
        "propose(new y, refines(x), feature, \"b\")".to_owned(),
        "supersede(x, by(y))".to_owned(),
        format!("place({answer}, y, defines)"),
        format!("sense({answer}, decision, specific, firm, explicit)"),
        format!("link({answer}, aaaaaaaa-0000-4000-8000-000000000002.0, answers)"),
        format!("conversational({})", Applying::answer(1)),
    ]
    .join("\n");
    applying.applied(&directives)?;
    let message = applying.run(&["show", Applying::ANSWER])?;
    let wanted = [
        format!("  place {answer} defines n3 current entered apply"),
        format!("  sense {answer} decision specific firm explicit current entered apply"),
        format!("  link {answer} answers aaaaaaaa-0000-4000-8000-000000000002.0 current entered apply"),
        format!("  conversational {} current entered apply", Applying::answer(1)),
    ];
    assert_eq!(Lines::missing(&message, &wanted), Vec::<&String>::new());
    let node = applying.run(&["show", "n3"])?;
    let wanted = [
        "n3 demo feature vague core entered apply \"b\"",
        "  refines n2 entered apply",
        "  supersedes n2 entered apply",
    ];
    assert_eq!(Lines::missing(&node, &wanted), Vec::<&&str>::new());
    Ok(())
}

#[test]
fn a_corrected_sense_supersedes_the_earlier_one_and_both_stay_stored() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let answer = Applying::answer(0);
    applying.applied(&format!("sense({answer}, decision, specific, firm, explicit)"))?;
    applying.applied(&format!("sense({answer}, opinion, general, tentative, implicit)"))?;
    let shown = applying.run(&["show", Applying::ANSWER])?;
    let wanted = [
        format!("  sense {answer} decision specific firm explicit superseded entered apply"),
        format!("  sense {answer} opinion general tentative implicit current entered apply"),
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&String>::new());
    assert_eq!(shown.matches("  sense ").count(), 2);
    Ok(())
}

#[test]
fn a_node_proposed_in_a_file_is_placed_later_in_it_by_label_or_by_its_id() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let directives = [
        "propose(new, refines(n1), feature, \"a\")".to_owned(),
        "propose(new b, refines(n2), feature, \"b\")".to_owned(),
        format!("place({}, n2, defines)", Applying::answer(0)),
        format!("place({}, b, decides)", Applying::answer(1)),
    ]
    .join("\n");
    let out = applying.applied(&directives)?;
    assert!(out.contains("proposed n3 by propose(new b, refines(n2), feature, \"b\")\n"));
    let shown = applying.run(&["show", Applying::ANSWER])?;
    let wanted = [
        format!("  place {} defines n2 current entered apply", Applying::answer(0)),
        format!("  place {} decides n3 current entered apply", Applying::answer(1)),
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&String>::new());
    let refusals = [
        "place(aaaaaaaa-0000-4000-8000-000000000003.0, c, defines)",
        "propose(new first, refines(n1), feature, \"again\")",
    ]
    .map(|directives| {
        applying.applied(&format!("propose(new first, refines(n1), feature, \"x\")\n{directives}")).err()
    });
    assert!(matches!(
        refusals,
        [Some(Failure::Unresolved(Unresolved::Unbound(_))), Some(Failure::Unresolved(Unresolved::Rebound(_)))]
    ));
    Ok(())
}

#[test]
fn apply_reads_its_file_from_the_working_directory_and_names_its_outcome() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.files.create_directories(Path::new("/work/notes"))?;
    applying.files.write(Path::new("/work/notes/directives"), Bytes::default())?;
    assert!(matches!(applying.run(&["apply", "directives"]), Err(Failure::Directives(_))));
    let lent = Cli::new(&Invocation::environment(), FakeLent::new(&applying.files), |_path| applying.store.clone())
        .rooted(["apply", "notes/../notes/directives"].iter().map(OsString::from), &mut Vec::new())?;
    assert_eq!(lent, Outcome::Applied);
    Ok(())
}

impl Applying {
    fn rooted(&self, directives: Bytes) -> Result<Answered, Failure> {
        self.files.create_directories(Path::new("/work/notes")).ok();
        self.files.write(Path::new("/work/notes/directives"), directives).ok();
        let mut out = Vec::new();
        let outcome = Cli::new(&Invocation::environment(), FakeLent::new(&self.files), |_path| self.store.clone())
            .rooted(["apply", "notes/directives"].iter().map(OsString::from), &mut out)?;
        Ok(Answered { outcome, out: String::from_utf8(out).unwrap_or_default() })
    }

    fn graph(&self) -> Result<[String; 2], Failure> {
        Ok([self.run(&["show", "n1"])?, self.run(&["show", Applying::ANSWER])?])
    }

    fn twice() -> String {
        let answer = Applying::answer(0);
        format!(
            "propose(new, refines(n1), feature, \"once\")\nplace({answer}, n2, defines)\nsense({answer}, decision, specific, firm, explicit)"
        )
    }
}

#[test]
fn a_file_applied_twice_writes_nothing_the_second_time_and_names_the_earlier_apply() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let first = applying.rooted(Bytes::from(&Applying::twice()))?;
    assert_eq!(first.outcome, Outcome::Applied);
    let before = applying.graph()?;
    let second = applying.rooted(Bytes::from(&Applying::twice()))?;
    assert!(matches!(
        &second.outcome,
        Outcome::AlreadyApplied { earlier }
            if earlier.identity() == AppliedId::new(1)
                && earlier.stamp().to_string().len() == "2026-11-06T00:00:00Z".len()
                && second.out == format!("already applied as file 1 at {}; nothing written\n", earlier.stamp())
    ));
    assert_eq!(applying.graph()?, before);
    assert!(matches!(applying.run(&["show", "n3"]), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn a_file_one_byte_away_from_an_applied_one_applies() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.rooted(Bytes::from(&Applying::twice()))?;
    let changed = applying.rooted(Bytes::from(&format!("{}\n", Applying::twice())))?;
    assert_eq!(changed.outcome, Outcome::Applied);
    assert!(changed.out.contains("proposed n3 by propose(new, refines(n1), feature, \"once\")\n"));
    assert_eq!(applying.run(&["show", "n1"])?.matches("  child ").count(), 2);
    Ok(())
}

#[test]
fn a_file_refused_in_resolution_is_refused_again_and_its_fix_applies() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let broken = format!("place({}, n9, defines)", Applying::answer(0));
    let refusals = [(); 2].map(|()| applying.rooted(Bytes::from(&broken)).err());
    assert!(matches!(
        refusals,
        [Some(Failure::Unresolved(Unresolved::Unplaced(_))), Some(Failure::Unresolved(Unresolved::Unplaced(_)))]
    ));
    let fixed = applying.rooted(Bytes::from(&format!("place({}, n1, defines)", Applying::answer(0))))?;
    assert_eq!(fixed.outcome, Outcome::Applied);
    assert!(applying.run(&["show", Applying::ANSWER])?.contains("defines n1 current entered apply\n"));
    assert!(matches!(applying.rooted(Bytes::new(vec![0xff])), Err(Failure::Directives(_))));
    Ok(())
}

#[test]
fn only_init_consumes_a_project() {
    let store = Sqlite::in_memory::<StreamSchema>();
    let session = Session::typical();
    let refused = [
        Invocation::against(&store, &session, &["show", "n1", "--consumes", "demo"]),
        Invocation::against(&store, &session, &["search", "thanks", "--consumes", "demo"]),
        Invocation::against(&store, &session, &["ingest", "--consumes", "demo"]),
        Invocation::against(&store, &session, &["apply", "directives", "--consumes", "demo"]),
        Invocation::against(&store, &session, &["--consumes", "demo", "init", "other"]),
        Invocation::against(&store, &session, &["init", "other", "--consumes", "demo", "--consumes", "demo"]),
    ];
    assert!(refused.iter().all(|refused| matches!(refused, Err(Failure::Usage))));
}

laws::law!(show_of_a_node_lists_its_ancestors_and_its_children, select(Facet::VARIANTS), facet, {
    let store = Sqlite::in_memory::<StreamSchema>();
    let shown = Graph::planted(&store, facet).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let wanted = [
        format!("n3 demo {facet} vague core entered init"),
        "  ancestor n1 demo project vague core entered init".to_owned(),
        format!("  ancestor n2 demo {facet} vague core entered init"),
        format!("  child n5 demo {facet} vague core entered init"),
        "  refines n2 entered init".to_owned(),
    ];
    let missing: Vec<&String> = wanted.iter().filter(|line| shown.matches(line.as_str()).count() == 0).collect();
    prop_assert_eq!(missing, Vec::<&String>::new());
    prop_assert_eq!(shown.matches("  child ").count(), 1);
    prop_assert_eq!(shown.matches("  ancestor ").count(), 2);
});

impl Graph {
    fn planted(store: &Sqlite, facet: Facet) -> Result<String, Failure> {
        Graph::init(store, &["demo"])?;
        Graph::refined(store, facet, &[1, 2, 2, 3])?;
        Graph::shown(store, "n3")
    }

    fn refined(store: &Sqlite, facet: Facet, parents: &[i64]) -> Result<Written, Failure> {
        let run = Run::root();
        let port = run.port(store.clone());
        let written = (2_i64..)
            .zip(parents)
            .map(|(child, parent)| {
                let entry = Entry::new("demo".parse()?, facet, Entered::Init.into());
                let planted = port.write(Planting::new(Some(Node::new(NodeId::new(child), entry)), None))?;
                let linked = port.write(Linked::new(
                    NodeId::new(child),
                    NodeRelation::Refines,
                    NodeId::new(*parent),
                    Entered::Init.into(),
                ))?;
                Ok(planted.plus(linked))
            })
            .collect::<Result<Vec<Written>, Failure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

#[derive(Clone, Copy, Debug)]
struct Asked(Stage, Lifecycle);

#[derive(Debug, PartialEq, Eq)]
struct Refused(NodeId, Stage);

#[derive(Clone, Copy, Debug)]
enum Lifecycle {
    Refine,
    Specify,
    Reopen,
    Drop,
}

impl Lifecycle {
    fn words(self) -> &'static [&'static str] {
        match self {
            Lifecycle::Refine => &["refine", "n2"],
            Lifecycle::Specify => {
                &["specify", "n2", "--artifact", "lint:no-comments", "--condition", "every comment is refused"]
            }
            Lifecycle::Reopen => &["reopen", "n2"],
            Lifecycle::Drop => &["drop", "n2", "--reason", "folded into n1"],
        }
    }

    fn path(stage: Stage) -> &'static [Lifecycle] {
        match stage {
            Stage::Vague => &[],
            Stage::Refining => &[Lifecycle::Refine],
            Stage::Terminal => &[Lifecycle::Refine, Lifecycle::Specify],
            Stage::Dropped => &[Lifecycle::Drop],
        }
    }

    fn reached(stage: Stage) -> Result<Applying, Failure> {
        let applying = Applying::ready()?;
        applying.applied("propose(new, refines(n1), feature, \"child\")")?;
        Lifecycle::path(stage).iter().try_for_each(|step| applying.outcome(step.words()).map(|_outcome| ()))?;
        Ok(applying)
    }

    fn moved(Asked(stage, command): Asked) -> Result<Outcome, Option<Refused>> {
        let applying = Lifecycle::reached(stage).map_err(|_unreached| None)?;
        applying.outcome(command.words()).map_err(Lifecycle::refused)
    }

    fn refused(failure: Failure) -> Option<Refused> {
        match failure {
            Failure::Misstaged { node, stage } => Some(Refused(node, stage)),
            _other => None,
        }
    }
}

impl Applying {
    fn outcome(&self, arguments: &[&str]) -> Result<Outcome, Failure> {
        Cli::new(&Invocation::environment(), FakeLent::new(&self.files), |_path| self.store.clone())
            .rooted(arguments.iter().map(OsString::from), &mut Vec::new())
    }
}

laws::table!(
    each_lifecycle_command_moves_only_from_its_allowed_stages_and_is_refused_naming_the_node_s_stage_elsewhere,
    |asked: &Asked| Lifecycle::moved(*asked),
    [
        Asked(Stage::Vague, Lifecycle::Refine) => Ok(Outcome::Moved(Stage::Refining)),
        Asked(Stage::Refining, Lifecycle::Refine) => Err(Some(Refused(NodeId::new(2), Stage::Refining))),
        Asked(Stage::Terminal, Lifecycle::Refine) => Err(Some(Refused(NodeId::new(2), Stage::Terminal))),
        Asked(Stage::Dropped, Lifecycle::Refine) => Err(Some(Refused(NodeId::new(2), Stage::Dropped))),
        Asked(Stage::Vague, Lifecycle::Specify) => Err(Some(Refused(NodeId::new(2), Stage::Vague))),
        Asked(Stage::Refining, Lifecycle::Specify) => Ok(Outcome::Moved(Stage::Terminal)),
        Asked(Stage::Terminal, Lifecycle::Specify) => Err(Some(Refused(NodeId::new(2), Stage::Terminal))),
        Asked(Stage::Dropped, Lifecycle::Specify) => Err(Some(Refused(NodeId::new(2), Stage::Dropped))),
        Asked(Stage::Vague, Lifecycle::Reopen) => Err(Some(Refused(NodeId::new(2), Stage::Vague))),
        Asked(Stage::Refining, Lifecycle::Reopen) => Err(Some(Refused(NodeId::new(2), Stage::Refining))),
        Asked(Stage::Terminal, Lifecycle::Reopen) => Ok(Outcome::Moved(Stage::Refining)),
        Asked(Stage::Dropped, Lifecycle::Reopen) => Ok(Outcome::Moved(Stage::Refining)),
        Asked(Stage::Vague, Lifecycle::Drop) => Ok(Outcome::Moved(Stage::Dropped)),
        Asked(Stage::Refining, Lifecycle::Drop) => Ok(Outcome::Moved(Stage::Dropped)),
        Asked(Stage::Terminal, Lifecycle::Drop) => Ok(Outcome::Moved(Stage::Dropped)),
        Asked(Stage::Dropped, Lifecycle::Drop) => Err(Some(Refused(NodeId::new(2), Stage::Dropped))),
    ]
);

#[test]
fn a_refused_transition_leaves_the_node_as_it_was() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Terminal)?;
    let before = applying.run(&["show", "n2"])?;
    assert!(applying.run(Lifecycle::Specify.words()).is_err());
    assert!(applying.run(Lifecycle::Refine.words()).is_err());
    assert_eq!(applying.run(&["show", "n2"])?, before);
    Ok(())
}

#[test]
fn specify_without_both_an_artifact_and_a_condition_fails_to_parse() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Refining)?;
    let refused = [
        applying.run(&["specify", "n2", "--artifact", "lint:no-comments"]),
        applying.run(&["specify", "n2", "--condition", "every comment is refused"]),
        applying.run(&["specify", "n2"]),
        applying.run(&["specify", "n2", "--artifact", "lint:no-comments", "--condition"]),
        applying.run(&["specify", "--artifact", "lint:no-comments", "--condition", "every comment is refused"]),
    ];
    assert!(refused.iter().all(|refused| matches!(refused, Err(Failure::Usage))));
    assert!(applying.run(&["show", "n2"])?.contains("n2 demo feature refining core entered apply"));
    Ok(())
}

#[test]
fn an_unparsed_artifact_condition_or_reason_is_refused_as_such() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Refining)?;
    let unspecified = applying.run(&["specify", "n2", "--artifact", "rule:x", "--condition", "it holds"]);
    let unstated = applying.run(&["specify", "n2", "--artifact", "lint:x", "--condition", "it\nholds"]);
    let unreasoned = applying.run(&["drop", "n2", "--reason", " "]);
    assert!(matches!(unspecified, Err(Failure::Unspecified(_))));
    assert!(matches!(unstated, Err(Failure::Unstated(_))));
    assert!(matches!(unreasoned, Err(Failure::Unstated(_))));
    assert!(applying.run(&["show", "n2"])?.contains("n2 demo feature refining core entered apply"));
    Ok(())
}

#[test]
fn each_lifecycle_option_belongs_to_its_own_command() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Vague)?;
    let refused = [
        applying.run(&["refine", "n2", "--reason", "no"]),
        applying.run(&["reopen", "n2", "--artifact", "lint:x"]),
        applying.run(&["drop", "n2", "--condition", "it holds"]),
        applying.run(&["drop", "n2", "--reason", "one", "--reason", "two"]),
        applying.run(&["specify", "n2", "--artifact", "lint:x", "--condition", "it holds", "--reason", "no"]),
        applying.run(&["specify", "n2", "--artifact", "lint:x", "--artifact", "lint:y", "--condition", "it holds"]),
        applying.run(&["role", "n2", "core", "--reason", "no"]),
        applying.run(&["role", "n2"]),
        applying.run(&["role", "n2", "chief"]),
        applying.run(&["show", "n2", "--artifact", "lint:x"]),
        applying.run(&["init", "other", "--reason", "no"]),
        applying.run(&["search", "thanks", "--condition", "it holds"]),
        applying.run(&["refine", "n2", "--dry-run"]),
    ];
    assert!(refused.iter().all(|refused| matches!(refused, Err(Failure::Usage))));
    assert!(applying.run(&["show", "n2"])?.contains("n2 demo feature vague core entered apply"));
    Ok(())
}

#[test]
fn show_renders_a_terminal_node_with_its_artifact_and_condition() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Terminal)?;
    let shown = applying.run(&["show", "n2"])?;
    let wanted = [
        "n2 demo feature terminal core entered apply \"child\"",
        "  artifact lint:no-comments",
        "  condition \"every comment is refused\"",
        "  ancestor n1 demo project vague core entered init",
        "  refines n1 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&&str>::new());
    assert_eq!(shown.matches('\n').count(), wanted.len());
    let reopened = applying.run(&["reopen", "n2"])?;
    assert!(reopened.contains("n2 demo feature refining core entered apply \"child\"\n  ancestor "));
    Ok(())
}

#[test]
fn a_dropped_node_stays_resolvable_with_its_reason() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Dropped)?;
    let shown = applying.run(&["show", "n2"])?;
    let wanted = [
        "n2 demo feature dropped core entered apply \"child\"",
        "  reason \"folded into n1\"",
        "  ancestor n1 demo project vague core entered init",
        "  refines n1 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&&str>::new());
    assert_eq!(shown.matches('\n').count(), wanted.len());
    assert!(applying.run(&["show", "n1"])?.contains("  child n2 demo feature dropped core entered apply \"child\"\n"));
    let placed = applying.applied(&format!("place({}, n2, mentions)", Applying::answer(1)))?;
    assert!(placed.contains("written 2\n"));
    Ok(())
}

#[test]
fn a_lifecycle_command_on_an_unknown_node_fails_naming_it() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    assert!(matches!(applying.run(&["refine", "n9"]), Err(Failure::Unplaced(_))));
    assert!(matches!(applying.run(&["role", "n9", "secondary"]), Err(Failure::Unplaced(_))));
    assert!(matches!(applying.run(&["refine", "o1"]), Err(Failure::Unnumbered(_))));
    Ok(())
}

#[test]
fn a_lifecycle_command_is_refused_while_the_store_lock_is_held() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Vague)?;
    let holder = Run::root();
    let port = holder.port(applying.store.clone());
    let held = port.lock()?;
    assert!(matches!(applying.run(&["refine", "n2"]), Err(Failure::Busy(_))));
    assert!(matches!(applying.run(&["role", "n2", "secondary"]), Err(Failure::Busy(_))));
    drop(held);
    assert_eq!(applying.outcome(&["refine", "n2"])?, Outcome::Moved(Stage::Refining));
    Ok(())
}

#[test]
fn a_secondary_parent_never_takes_a_core_child_and_the_refusal_names_the_edge() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Vague)?;
    let edge = NodeEdge::new(NodeId::new(2), NodeRelation::Refines, NodeId::new(1));
    let parent = applying.run(&["role", "n1", "secondary"]);
    assert!(matches!(parent, Err(Failure::Inverted { edge: refused }) if refused == edge));
    assert!(applying.run(&["show", "n1"])?.contains("n1 demo project vague core entered init\n"));
    assert_eq!(applying.outcome(&["role", "n2", "secondary"])?, Outcome::Recast(Role::Secondary));
    assert_eq!(applying.outcome(&["role", "n1", "secondary"])?, Outcome::Recast(Role::Secondary));
    let child = applying.run(&["role", "n2", "core"]);
    assert!(matches!(child, Err(Failure::Inverted { edge: refused }) if refused == edge));
    let child = applying.run(&["show", "n2"])?;
    assert!(child.contains("n2 demo feature vague secondary entered apply"));
    assert!(child.contains("  role secondary entered role\n"));
    Ok(())
}

#[test]
fn a_proposal_inherits_a_secondary_parent_s_role_and_records_the_inheritance() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.run(&["role", "n1", "secondary"])?;
    applying.applied("propose(new, refines(n1), feature, \"child\")")?;
    let child = applying.run(&["show", "n2"])?;
    assert!(child.contains("n2 demo feature vague secondary entered apply"));
    assert!(child.contains("  role secondary entered inheritance\n"));
    assert_eq!(applying.outcome(&["role", "n1", "core"])?, Outcome::Recast(Role::Core));
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Ruled {
    parent: Role,
    child: Role,
}

impl Ruled {
    fn recast(self) -> Result<Roles, Option<NodeEdge>> {
        let applying = Applying::ready().map_err(|_unready| None)?;
        applying.outcome(&["role", "n1", self.parent.name()]).map_err(|_unrecast| None)?;
        applying.applied("propose(new, refines(n1), feature, \"metering\")").map_err(|_unproposed| None)?;
        let proposed = applying.role(NodeId::new(2)).ok_or(None)?;
        applying.outcome(&["role", "n2", self.child.name()]).map_err(Ruled::inverted)?;
        applying.role(NodeId::new(2)).map(|recast| Roles { proposed, recast }).ok_or(None)
    }

    fn inverted(failure: Failure) -> Option<NodeEdge> {
        match failure {
            Failure::Inverted { edge } => Some(edge),
            _other => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Roles {
    proposed: Role,
    recast: Role,
}

impl Applying {
    fn role(&self, node: NodeId) -> Option<Role> {
        let run = Run::root();
        let placed = run.port(self.store.clone()).read(ShownNode::from(node)).ok().flatten();
        placed.map(|placed| placed.node().casting().role())
    }
}

laws::table!(
    a_core_parent_may_take_a_secondary_child_and_a_secondary_parent_never_a_core_one,
    |ruled: &Ruled| Ruled::recast(*ruled),
    [
        Ruled { parent: Role::Core, child: Role::Secondary } => Ok(Roles { proposed: Role::Core, recast: Role::Secondary }),
        Ruled { parent: Role::Core, child: Role::Core } => Ok(Roles { proposed: Role::Core, recast: Role::Core }),
        Ruled { parent: Role::Secondary, child: Role::Secondary } => Ok(Roles { proposed: Role::Secondary, recast: Role::Secondary }),
        Ruled { parent: Role::Secondary, child: Role::Core } => Err(Some(NodeEdge::new(NodeId::new(2), NodeRelation::Refines, NodeId::new(1)))),
    ]
);

struct Workspace {
    store: Sqlite,
    files: Memory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unplanted {
    Inverted,
    Unrooted,
}

struct Replanted {
    refused: Option<Unplanted>,
    unplaced: Option<NodeId>,
    config: Kind,
    out: String,
}

impl Unplanted {
    fn consumed(self, workspace: &Workspace) -> Result<&'static str, Failure> {
        match self {
            Unplanted::Inverted => workspace.run("/p", &["role", "n1", "secondary"]).map(|_recast| "p"),
            Unplanted::Unrooted => Ok("nowhere"),
        }
    }

    fn seen(refused: Result<String, Failure>) -> Option<Unplanted> {
        match refused {
            Err(Failure::Inverted { .. }) => Some(Unplanted::Inverted),
            Err(Failure::Unrooted(_)) => Some(Unplanted::Unrooted),
            _other => None,
        }
    }

    fn unplaced(shown: Result<String, Failure>) -> Option<NodeId> {
        match shown {
            Err(Failure::Unplaced(node)) => Some(node),
            _other => None,
        }
    }
}

impl Workspace {
    fn split() -> Workspace {
        let files = Session::typical().files();
        files.create_directories(Path::new("/p/.git")).ok();
        files.create_directories(Path::new("/q/.git")).ok();
        Workspace { store: Sqlite::in_memory::<StreamSchema>(), files }
    }

    fn run(&self, directory: &str, arguments: &[&str]) -> Result<String, Failure> {
        let run = Run::root();
        let mut out = Vec::new();
        Cli::new(&Invocation::at(directory), FakeLent::new(&self.files), |_path| self.store.clone()).report(
            &run,
            arguments.iter().map(OsString::from),
            &mut out,
        )?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }

    fn replanted(unplanted: Unplanted) -> Result<Replanted, Failure> {
        let workspace = Workspace::split();
        workspace.run("/p", &["init", "p"])?;
        let consumed = unplanted.consumed(&workspace)?;
        let refused = Unplanted::seen(workspace.run("/q", &["init", "q", "--consumes", consumed]));
        let unplaced = Unplanted::unplaced(workspace.run("/q", &["show", "n2"]));
        let config = workspace.files.kind(Path::new("/q/.meta/local.toml"));
        let replanted = workspace.run("/q", &["init", "q"])?;
        Ok(Replanted { refused, unplaced, config, out: replanted })
    }
}

laws::law!(
    a_refused_init_plants_no_node_and_keeps_no_local_config_and_a_later_init_succeeds,
    select(vec![Unplanted::Inverted, Unplanted::Unrooted]),
    unplanted,
    {
        let replanted = Workspace::replanted(unplanted).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        prop_assert_eq!(replanted.refused, Some(unplanted));
        prop_assert_eq!(replanted.unplaced, Some(NodeId::new(2)));
        prop_assert_eq!(replanted.config, Kind::Missing);
        prop_assert_eq!(replanted.out, "root n2 project q\n");
    }
);

#[derive(Clone, Copy, Debug)]
enum Spoken {
    Ingest,
    Show,
    Search,
    Init,
    Apply,
    Refine,
    Specify,
    Reopen,
    Drop,
    Role,
}

#[derive(Clone, Copy, Debug)]
struct Foreign(Spoken, &'static str);

#[derive(Debug, PartialEq, Eq)]
enum Parsed {
    Usage,
    Otherwise,
}

impl Spoken {
    fn words(self) -> &'static [&'static str] {
        match self {
            Spoken::Ingest => &["ingest"],
            Spoken::Show => &["show"],
            Spoken::Search => &["search"],
            Spoken::Init => &["init"],
            Spoken::Apply => &["apply"],
            Spoken::Refine => &["refine"],
            Spoken::Specify => &["specify", "--artifact", "lint:x", "--condition", "it holds"],
            Spoken::Reopen => &["reopen"],
            Spoken::Drop => &["drop", "--reason", "folded"],
            Spoken::Role => &["role", "n1"],
        }
    }

    fn missing(self) -> &'static str {
        match self {
            Spoken::Ingest | Spoken::Init => "demo",
            Spoken::Show | Spoken::Refine | Spoken::Specify | Spoken::Reopen | Spoken::Drop => "n1",
            Spoken::Search => "thanks",
            Spoken::Apply => "notes/directives",
            Spoken::Role => "core",
        }
    }
}

impl Foreign {
    fn parsed(&self) -> Parsed {
        let Foreign(spoken, flag) = *self;
        let arguments = [spoken.words(), &[flag, spoken.missing()]].concat();
        Some(Invocation::output(&Session::typical(), &arguments))
            .filter(|parsed| matches!(parsed, Err(Failure::Usage)))
            .map_or(Parsed::Otherwise, |_usage| Parsed::Usage)
    }
}

laws::table!(
    each_command_refuses_every_flag_it_does_not_own_as_usage_never_as_a_positional,
    Foreign::parsed,
    [
        Foreign(Spoken::Ingest, "--all") => Parsed::Usage,
        Foreign(Spoken::Ingest, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Ingest, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Ingest, "--condition") => Parsed::Usage,
        Foreign(Spoken::Ingest, "--reason") => Parsed::Usage,
        Foreign(Spoken::Ingest, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Show, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Show, "--all") => Parsed::Usage,
        Foreign(Spoken::Show, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Show, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Show, "--condition") => Parsed::Usage,
        Foreign(Spoken::Show, "--reason") => Parsed::Usage,
        Foreign(Spoken::Show, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Search, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Search, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Search, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Search, "--condition") => Parsed::Usage,
        Foreign(Spoken::Search, "--reason") => Parsed::Usage,
        Foreign(Spoken::Search, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Init, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Init, "--all") => Parsed::Usage,
        Foreign(Spoken::Init, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Init, "--condition") => Parsed::Usage,
        Foreign(Spoken::Init, "--reason") => Parsed::Usage,
        Foreign(Spoken::Init, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Apply, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Apply, "--all") => Parsed::Usage,
        Foreign(Spoken::Apply, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Apply, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Apply, "--condition") => Parsed::Usage,
        Foreign(Spoken::Apply, "--reason") => Parsed::Usage,
        Foreign(Spoken::Apply, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Refine, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Refine, "--all") => Parsed::Usage,
        Foreign(Spoken::Refine, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Refine, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Refine, "--condition") => Parsed::Usage,
        Foreign(Spoken::Refine, "--reason") => Parsed::Usage,
        Foreign(Spoken::Refine, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Specify, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Specify, "--all") => Parsed::Usage,
        Foreign(Spoken::Specify, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Specify, "--reason") => Parsed::Usage,
        Foreign(Spoken::Specify, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--all") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--condition") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--reason") => Parsed::Usage,
        Foreign(Spoken::Reopen, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Drop, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Drop, "--all") => Parsed::Usage,
        Foreign(Spoken::Drop, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Drop, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Drop, "--condition") => Parsed::Usage,
        Foreign(Spoken::Drop, "--frobnicate") => Parsed::Usage,
        Foreign(Spoken::Role, "--dry-run") => Parsed::Usage,
        Foreign(Spoken::Role, "--all") => Parsed::Usage,
        Foreign(Spoken::Role, "--consumes") => Parsed::Usage,
        Foreign(Spoken::Role, "--artifact") => Parsed::Usage,
        Foreign(Spoken::Role, "--condition") => Parsed::Usage,
        Foreign(Spoken::Role, "--reason") => Parsed::Usage,
        Foreign(Spoken::Role, "--frobnicate") => Parsed::Usage,
    ]
);

struct Rendered;

impl Rendered {
    fn applied(workspace: &Workspace, directives: &str) -> Result<String, Failure> {
        workspace.files.write(Path::new("/q/directives"), Bytes::from(directives)).ok();
        workspace.run("/q", &["apply", "directives"])
    }

    fn tree() -> Result<String, Box<dyn std::error::Error>> {
        let workspace = Workspace::split();
        workspace.run("/p", &["init", "upstream"])?;
        workspace.run("/q", &["init", "demo", "--consumes", "upstream"])?;
        Rendered::applied(
            &workspace,
            "propose(new, refines(n2), feature, \"old\")\npropose(new, refines(n2), feature, \"new\")\npropose(new, refines(n2), feature, \"spare\")",
        )?;
        Rendered::applied(&workspace, "supersede(n3, by(n4))")?;
        workspace.run("/q", &["refine", "n3"])?;
        workspace.run("/q", &["refine", "n4"])?;
        workspace.run(
            "/q",
            &["specify", "n4", "--artifact", "lint:no-comments", "--condition", "every comment is refused"],
        )?;
        workspace.run("/q", &["role", "n5", "secondary"])?;
        Rendered::applied(&workspace, "propose(new, refines(n5), smell, \"inherited\")")?;
        workspace.run("/q", &["drop", "n5", "--reason", "folded into n4"])?;
        let refused = workspace.run("/q", &["role", "n2", "secondary"]).err().map(|failure| format!("{failure}\n"));
        let shown = ["n1", "n2", "n3", "n4", "n5", "n6"]
            .iter()
            .map(|node| workspace.run("/q", &["show", node]).map(|out| format!("design show {node}\n{out}")))
            .collect::<Result<String, Failure>>()?;
        Ok(format!("{shown}design role n2 secondary\n{}", refused.unwrap_or_default()))
    }
}

#[test]
fn show_renders_every_stage_edge_and_role_of_a_small_tree() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = Rendered::tree()?;
    golden::golden!("show.txt", &rendered)?;
    Ok(())
}
