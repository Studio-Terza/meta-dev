mod notes;
mod structure;
mod view;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use std::sync::Mutex;

use std::str::FromStr;

use console_ports::{Heard, Variables};

use design::{
    Anchored, Attempted, Cli, Count, Failure, Flailing, Lapsed, Line, Miss, Noted, Outcome, Reconciliation, Reference,
    Remark, Summary, Unmatched, Unresolved, Viewed,
};
use design_model::{
    AnyStage, Artifact, Basis, Casting, Condition, Confidence, Dropped, Entered, Entry, Explicitness, Facet, JobId,
    Kin, Node, NodeEdge, NodeId, NodeRelation, NotesLine, Ordinal, Outline, Prefix, Project, Provenance, Reason,
    Refining, Role, RunId, SegmentRelation, Sense, SenseKind, SpanRelation, Specificity, Stage, Standing, Terminal,
    Title, Vague,
};
use design_store::{
    AppliedId, Excerpt, Linked, Planting, RelationName, Resolved, Resolving, RunState, ShapeName, ShownMessage,
    ShownNode, Stored, StreamSchema,
};
use dispatch_ports::{Tokens, Usage};
use file_ports::{Bytes, FileFailure, Files, Kind, Memory, Offset, Size, Text};
use grammar::notes::{Cite, Quote};
use laws::proptest;
use laws::proptest::prelude::{Just, any, prop_assert, prop_assert_eq, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::{BoxedStrategy, Strategy};
use laws::proptest::test_runner::TestCaseError;
use newtype::{Counter, Newtype};
use port_core::{Held, Pass, Release, ReleaseReceipt, Released, Resource, Run, port};
use store_ports::{Busy, Sqlite, Store, Written};
use stream::{Relation, Shape};
use structure::Staff;
use structurer::{Directive, Directives, Lapse, NodeRef, Proposal, SegmentRef, Uncovered};
use strum::VariantArray;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, Uuid};

const HOME: &str = "/home/tester";

#[derive(Clone, Copy, Debug)]
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
        Session::stamped(speaker, uuid, parent, text, "2026-01-01")
    }

    fn stamped(speaker: Speaker, uuid: &str, parent: &str, text: &str, day: &str) -> String {
        Session::recorded("s", speaker, uuid, [parent, text, day])
    }

    fn recorded(session: &str, speaker: Speaker, uuid: &str, [parent, text, day]: [&str; 3]) -> String {
        let Wire { kind, origin } = speaker.wire();
        let content = speaker.content(text);
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":{parent},\"sessionId\":\"{session}\",\"timestamp\":\"{day}T00:00:00Z\",\"origin\":{origin},\"message\":{{\"model\":\"claude\",\"content\":{content}}}}}\n"
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
                Self::stamped(
                    Speaker::Owner,
                    Self::ASKED,
                    "null",
                    "what of aaaaaaaa-0000-4000-8000-000000000004",
                    "2026-01-01",
                ),
                Self::stamped(
                    Speaker::Lead,
                    "aaaaaaaa-0000-4000-8000-000000000002",
                    "\"aaaaaaaa-0000-4000-8000-000000000001\"",
                    "1. a\\n2. b",
                    "2026-01-02",
                ),
                Self::stamped(
                    Speaker::Unheard,
                    "b1",
                    "\"aaaaaaaa-0000-4000-8000-000000000002\"",
                    "result",
                    "2026-01-03",
                ),
                Self::stamped(
                    Speaker::Owner,
                    "aaaaaaaa-0000-4000-8000-000000000003",
                    "\"b1\"",
                    "1. yes\\n2. no",
                    "2026-01-04",
                ),
                Self::stamped(
                    Speaker::Lead,
                    "aaaaaaaa-0000-4000-8000-000000000004",
                    "\"aaaaaaaa-0000-4000-8000-000000000003\"",
                    "see aaaaaaaa-0000-4000-8000-000000000001",
                    "2026-01-05",
                ),
                Self::stamped(
                    Speaker::Owner,
                    "aaaaaaaa-0000-4000-8000-000000000005",
                    "\"aaaaaaaa-0000-4000-8000-000000000004\"",
                    "what of aaaaaaaa-0000-4000-8000-000000000004",
                    "2026-01-06",
                ),
            ]
            .concat(),
        )
    }

    fn distinct() -> Session {
        Session::working(
            [
                Self::line(Speaker::Owner, "abcd0001-0000-4000-8000-000000000001", "null", "1. yes\\n2. no\\n3. later"),
                Self::line(Speaker::Lead, "bcde0002-0000-4000-8000-000000000002", "null", "1. a\\n2. b"),
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
            .collect::<Variables>()
            .hearing(Heard::new("{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}".to_owned()))
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
        Cli::new(&Self::environment(), files, |_path| store.clone(), Staff::idle()).report(
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
    assert!(out.contains("unrecognised \"brand-new-kind\" 1\n"));
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
                port.read(ShownMessage::named(uuid.clone()))?
                    .ok_or(Failure::Unmatched(Unmatched::NoMatch(Prefix::new(uuid.into_inner()))))
            })
            .collect::<Result<Vec<_>, Failure>>()?;
        Ok(shown
            .iter()
            .flat_map(Stored::links)
            .map(|link| {
                let [source, target] = link.anchors();
                let relation: &RelationName = link.as_ref();
                Link { source: source.to_string(), relation: relation.to_string(), target: target.to_string() }
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
        let outcome = Cli::new(&Invocation::environment(), session.files(), |_path| store.clone(), Staff::idle())
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
    assert!(out.contains("o1 owner session \"s\" at \"2026-01-01T00:00:00Z\""));
    assert!(out.contains("  segment o1.0 item "));
    Ok(())
}

#[test]
fn show_of_an_unknown_uuid_fails_naming_it() {
    let failed = Invocation::shown(&Session::typical(), "nobody");
    assert!(matches!(failed, Err(Failure::Unmatched(Unmatched::NoMatch(_)))));
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
    assert!(matches!(before, Err(Failure::Unmatched(Unmatched::NoMatch(_)))));
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
        Cli::new(
            &Invocation::at(directory),
            files,
            |path: PathBuf| {
                opened.lock().map(|mut paths| paths.push(path)).ok();
                Sqlite::in_memory::<StreamSchema>()
            },
            Staff::idle(),
        )
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
    assert!(Invocation::against(&store, &Session::answered(), &["show", "l1"])?.contains("l1 lead session \"s\""));
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
    prop_assert!(out.contains("hit o1.0 owner"));
    prop_assert!(!out.contains("hit o2.0 owner"));
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
    prop_assert!(out.contains("hit o1.0 owner"));
});

laws::law!(the_lead_is_searched_only_when_everyone_is_asked_for, "[a-z]{4,10}", word, {
    let session =
        Found::said(&[Said::by(Speaker::Owner, "o1", word.clone()), Said::by(Speaker::Lead, "l1", word.clone())]);
    let owner = Found::by(&session, &["search", &word])?;
    let everyone = Found::by(&session, &["search", "--all", &word])?;
    prop_assert!(owner.contains("hit o1.0 owner"));
    prop_assert!(!owner.contains("hit l1.0 lead"));
    prop_assert!(everyone.contains("hit o1.0 owner"));
    prop_assert!(everyone.contains("hit l1.0 lead"));
});

laws::law!(a_word_is_found_whatever_its_case, "[a-z]{4,10}", word, {
    let raised: String = word.chars().map(|letter| letter.to_ascii_uppercase()).collect();
    let session =
        Found::said(&[Said::by(Speaker::Owner, "o1", raised.clone()), Said::by(Speaker::Owner, "o2", word.clone())]);
    let lower = Found::by(&session, &["search", &word])?;
    let upper = Found::by(&session, &["search", &raised])?;
    prop_assert!(lower.contains("hit o1.0 owner") && lower.contains("hit o2.0 owner"));
    prop_assert!(upper.contains("hit o1.0 owner") && upper.contains("hit o2.0 owner"));
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
    assert!(opened.out.contains("root n1 project \"demo\""));
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
    assert!(consuming.contains("consumes n1 project \"meta-dev\""));
    assert!(
        Graph::shown(&store, "n2")?
            .contains("n2 \"demo\" project vague vague core entered init\n  edge n2 consumes n1 entered init\n")
    );
    assert!(
        Graph::shown(&store, "n1")?
            .contains("n1 \"meta-dev\" project vague vague core entered init\n  edge n2 consumes n1 entered init\n")
    );
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
        Applying::over(&Session::linked())
    }

    fn over(session: &Session) -> Result<Applying, Failure> {
        let applying = Applying { store: Sqlite::in_memory::<StreamSchema>(), files: session.files() };
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
        format!("applied \"place({answer}, n1, decides)\""),
        "proposed n2 by \"propose(new, refines(n1), feature, \\\"Apply \\\\\\\"directives\\\\\\\"\\\")\"".to_owned(),
        format!("applied \"sense({answer}, decision, specific, firm, explicit)\""),
        format!("applied \"link({answer}, aaaaaaaa-0000-4000-8000-000000000002.0, answers)\""),
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
        "n4 \"demo\" feature vague vague core entered apply \"echo\"",
        "  ancestor n1 \"demo\" project vague vague core entered init",
        "  ancestor n2 \"demo\" feature vague vague core entered apply \"apply\"",
        "  edge n4 refines n2 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&child, &wanted), Vec::<&&str>::new());
    assert_eq!(child.matches('\n').count(), wanted.len());
    let root = applying.run(&["show", "n1"])?;
    assert_eq!(root.matches("  child ").count(), 2);
    assert!(root.contains("  child n3 \"demo\" smell vague vague core entered apply \"drift\"\n"));
    Ok(())
}

#[test]
fn a_superseded_node_stays_resolvable() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.applied("propose(new, refines(n1), feature, \"old\")\npropose(new, refines(n1), feature, \"new\")")?;
    applying.applied("supersede(n2, by(n3))")?;
    let root = "  ancestor n1 \"demo\" project vague vague core entered init";
    let supersedes = "  edge n3 supersedes n2 entered apply";
    let cast = "  role core entered apply";
    let wanted = [
        (
            "n2",
            vec![
                "n2 \"demo\" feature vague vague core entered apply \"old\"",
                root,
                "  edge n2 refines n1 entered apply",
                supersedes,
                cast,
            ],
        ),
        (
            "n3",
            vec![
                "n3 \"demo\" feature vague vague core entered apply \"new\"",
                root,
                "  edge n3 refines n1 entered apply",
                supersedes,
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
    run.port(applying.store.clone()).write(Planting::sown(Node::new(NodeId::new(2), entry)))?;
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
        "propose(new y, refines(n1), feature, \"b\")".to_owned(),
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
        format!("  annotation place {answer} defines n3 current entered apply"),
        format!("  annotation sense {answer} decision specific firm explicit current entered apply"),
        format!("  annotation link {answer} answers aaaaaaaa-0000-4000-8000-000000000002.0 current entered apply"),
        format!("  annotation conversational {} current entered apply", Applying::answer(1)),
    ];
    assert_eq!(Lines::missing(&message, &wanted), Vec::<&String>::new());
    let node = applying.run(&["show", "n3"])?;
    let wanted = [
        "n3 \"demo\" feature vague vague core entered apply \"b\"",
        "  edge n3 refines n1 entered apply",
        "  edge n3 supersedes n2 entered apply",
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
        format!("  annotation sense {answer} decision specific firm explicit superseded entered apply"),
        format!("  annotation sense {answer} opinion general tentative implicit current entered apply"),
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&String>::new());
    assert_eq!(shown.matches("  annotation sense ").count(), 2);
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
    assert!(out.contains("proposed n3 by \"propose(new b, refines(n2), feature, \\\"b\\\")\"\n"));
    let shown = applying.run(&["show", Applying::ANSWER])?;
    let wanted = [
        format!("  annotation place {} defines n2 current entered apply", Applying::answer(0)),
        format!("  annotation place {} decides n3 current entered apply", Applying::answer(1)),
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
    let lent = Cli::new(
        &Invocation::environment(),
        FakeLent::new(&applying.files),
        |_path| applying.store.clone(),
        Staff::idle(),
    )
    .rooted(["apply", "notes/../notes/directives"].iter().map(OsString::from), &mut Vec::new())?;
    assert_eq!(lent, Outcome::Applied);
    Ok(())
}

impl Applying {
    fn rooted(&self, directives: Bytes) -> Result<Answered, Failure> {
        self.files.create_directories(Path::new("/work/notes")).ok();
        self.files.write(Path::new("/work/notes/directives"), directives).ok();
        let mut out = Vec::new();
        let outcome =
            Cli::new(&Invocation::environment(), FakeLent::new(&self.files), |_path| self.store.clone(), Staff::idle())
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
                && second.out == format!("already applied file 1 at \"{}\"\n", earlier.stamp())
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
    assert!(changed.out.contains("proposed n3 by \"propose(new, refines(n1), feature, \\\"once\\\")\"\n"));
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
        format!("n3 \"demo\" {facet} vague vague core entered init"),
        "  ancestor n1 \"demo\" project vague vague core entered init".to_owned(),
        format!("  ancestor n2 \"demo\" {facet} vague vague core entered init"),
        format!("  child n5 \"demo\" {facet} vague vague core entered init"),
        "  edge n3 refines n2 entered init".to_owned(),
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
                let planted = port.write(Planting::sown(Node::new(NodeId::new(child), entry)))?;
                let (child, parent) = (Graph::resolved(&port, child)?, Graph::resolved(&port, *parent)?);
                let linked = port.write(Linked::new(child, NodeRelation::Refines, parent, Entered::Init.into()))?;
                Ok(planted.plus(linked))
            })
            .collect::<Result<Vec<Written>, Failure>>()?;
        Ok(Written::total(written.into_iter()))
    }

    fn resolved(store: &impl Store, id: i64) -> Result<Resolved, Failure> {
        store.read(Resolving::from(NodeId::new(id)))?.ok_or(Failure::Unplaced(NodeId::new(id)))
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
        Cli::new(&Invocation::environment(), FakeLent::new(&self.files), |_path| self.store.clone(), Staff::idle())
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
    assert!(applying.run(&["show", "n2"])?.contains("n2 \"demo\" feature refining set core entered apply"));
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
    assert!(applying.run(&["show", "n2"])?.contains("n2 \"demo\" feature refining set core entered apply"));
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
    assert!(applying.run(&["show", "n2"])?.contains("n2 \"demo\" feature vague vague core entered apply"));
    Ok(())
}

#[test]
fn show_renders_a_terminal_node_with_its_artifact_and_condition() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Terminal)?;
    let shown = applying.run(&["show", "n2"])?;
    let wanted = [
        "n2 \"demo\" feature terminal set core entered apply \"child\"",
        "  artifact \"lint:no-comments\"",
        "  condition \"every comment is refused\"",
        "  ancestor n1 \"demo\" project vague vague core entered init",
        "  edge n2 refines n1 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&&str>::new());
    assert_eq!(shown.matches('\n').count(), wanted.len());
    let reopened = applying.run(&["reopen", "n2"])?;
    assert!(reopened.contains("n2 \"demo\" feature refining set core entered apply \"child\"\n  ancestor "));
    Ok(())
}

#[test]
fn a_dropped_node_stays_resolvable_with_its_reason() -> Result<(), Failure> {
    let applying = Lifecycle::reached(Stage::Dropped)?;
    let shown = applying.run(&["show", "n2"])?;
    let wanted = [
        "n2 \"demo\" feature dropped set core entered apply \"child\"",
        "  reason \"folded into n1\"",
        "  ancestor n1 \"demo\" project vague vague core entered init",
        "  edge n2 refines n1 entered apply",
        "  role core entered apply",
    ];
    assert_eq!(Lines::missing(&shown, &wanted), Vec::<&&str>::new());
    assert_eq!(shown.matches('\n').count(), wanted.len());
    assert!(
        applying
            .run(&["show", "n1"])?
            .contains("  child n2 \"demo\" feature dropped set core entered apply \"child\"\n")
    );
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
    assert!(applying.run(&["show", "n1"])?.contains("n1 \"demo\" project vague vague core entered init\n"));
    assert_eq!(applying.outcome(&["role", "n2", "secondary"])?, Outcome::Recast(Role::Secondary));
    assert_eq!(applying.outcome(&["role", "n1", "secondary"])?, Outcome::Recast(Role::Secondary));
    let child = applying.run(&["role", "n2", "core"]);
    assert!(matches!(child, Err(Failure::Inverted { edge: refused }) if refused == edge));
    let child = applying.run(&["show", "n2"])?;
    assert!(child.contains("n2 \"demo\" feature vague vague secondary entered apply"));
    assert!(child.contains("  role secondary entered role\n"));
    Ok(())
}

#[test]
fn a_proposal_inherits_a_secondary_parent_s_role_and_records_the_inheritance() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    applying.run(&["role", "n1", "secondary"])?;
    applying.applied("propose(new, refines(n1), feature, \"child\")")?;
    let child = applying.run(&["show", "n2"])?;
    assert!(child.contains("n2 \"demo\" feature vague vague secondary entered apply"));
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
        Cli::new(&Invocation::at(directory), FakeLent::new(&self.files), |_path| self.store.clone(), Staff::idle())
            .report(&run, arguments.iter().map(OsString::from), &mut out)?;
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
        prop_assert_eq!(replanted.out, "root n2 project \"q\"\n");
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

    fn placed(workspace: &Workspace) -> Result<String, Failure> {
        let transcripts = Path::new(HOME).join(".claude/projects/-q");
        workspace.files.create_directories(&transcripts).ok();
        workspace.files.write(&transcripts.join("s.jsonl"), Bytes::from(&Session::linked().lines)).ok();
        workspace.run("/q", &["ingest"])?;
        Rendered::applied(
            workspace,
            &[
                format!("place({}, n4, decides)", Applying::answer(0)),
                format!("sense({}, decision, specific, firm, explicit)", Applying::answer(0)),
                format!("place({}, n4, questions)", Applying::answer(1)),
            ]
            .join("\n"),
        )?;
        Rendered::applied(workspace, &format!("sense({}, opinion, general, tentative, implicit)", Applying::answer(0)))
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
        Rendered::placed(&workspace)?;
        Ok(["n1", "n2", "n3", "n4", "n5", "n6"]
            .iter()
            .map(|node| workspace.run("/q", &["show", node]).map(|out| format!("design show {node}\n{out}")))
            .collect::<Result<String, Failure>>()?)
    }
}

#[test]
fn show_renders_every_stage_edge_role_and_placement_of_a_small_tree() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = Rendered::tree()?;
    golden::golden!("show.txt", &rendered)?;
    Ok(())
}

#[test]
fn apply_echoes_every_directive_kind_and_show_cites_every_remarked_segment() -> Result<(), Box<dyn std::error::Error>> {
    let applying = Applying::over(&Session::distinct())?;
    let directives = [
        "place(abcd0001.0, n1, decides)",
        "sense(abcd0001.0, decision, specific, firm, explicit)",
        "link(abcd0001.1, bcde0002.0, answers)",
        "conversational(abcd0001.2)",
        "propose(new, refines(n1), feature, \"applied\")",
    ]
    .join("\n");
    let applied = applying.applied(&directives)?;
    let shown = applying.run(&["show", "abcd0001"])?;
    golden::golden!("apply.txt", &format!("design apply notes/directives\n{applied}design show abcd0001\n{shown}"))?;
    Ok(())
}

struct Generated;

impl Generated {
    fn text() -> BoxedStrategy<String> {
        any::<String>().boxed()
    }

    fn word() -> BoxedStrategy<String> {
        "[A-Za-z0-9_][A-Za-z0-9_-]{0,15}".boxed()
    }

    fn shape() -> BoxedStrategy<ShapeName> {
        prop_oneof![Generated::word(), select(Shape::VARIANTS).prop_map(|shape| shape.to_string())]
            .prop_map(ShapeName::new)
            .boxed()
    }

    fn relation() -> BoxedStrategy<RelationName> {
        prop_oneof![Generated::word(), select(Relation::VARIANTS).prop_map(|relation| relation.to_string())]
            .prop_map(RelationName::new)
            .boxed()
    }

    fn node() -> BoxedStrategy<NodeId> {
        (0..i64::MAX).prop_map(NodeId::new).boxed()
    }

    fn count() -> BoxedStrategy<Count> {
        any::<usize>().prop_map(Count::new).boxed()
    }

    fn author() -> BoxedStrategy<Authorship> {
        select(vec![Authorship::Owner, Authorship::Lead]).boxed()
    }

    fn entered() -> BoxedStrategy<Provenance> {
        prop_oneof![
            select(Entered::VARIANTS).prop_map(Provenance::from),
            any::<i64>().prop_map(|run| Provenance::Agent(RunId::new(run))),
            (1..=usize::MAX).prop_map(|line| Provenance::Import(NotesLine::new(line))),
        ]
        .boxed()
    }

    fn segment() -> BoxedStrategy<SegmentRef> {
        (Generated::message(), 0usize..100_000)
            .prop_map(|(prefix, ordinal)| SegmentRef::new(Prefix::new(prefix), Ordinal::new(ordinal)))
            .boxed()
    }

    fn anchored() -> BoxedStrategy<Anchored> {
        prop_oneof![
            Generated::segment().prop_map(Anchored::Segment),
            Generated::message().prop_map(|prefix| Anchored::Message(Prefix::new(prefix))),
        ]
        .boxed()
    }

    fn message() -> BoxedStrategy<String> {
        "[A-Za-z0-9_][A-Za-z0-9_-]{0,36}".boxed()
    }

    fn uuid() -> BoxedStrategy<Uuid> {
        Generated::message().prop_map(Uuid::new).boxed()
    }

    fn project() -> BoxedStrategy<Project> {
        "[a-z0-9][a-z0-9._-]{0,10}".prop_filter_map("a project name", |name| name.parse().ok()).boxed()
    }

    fn sentence<Spelled: FromStr + std::fmt::Debug + 'static>() -> BoxedStrategy<Spelled> {
        "[a-z][a-z \"]{0,20}[a-z]".prop_filter_map("one line of text", |text| text.parse().ok()).boxed()
    }

    fn artifact() -> BoxedStrategy<Artifact> {
        "(lint|hook|tool|behaviour):[a-z][a-z-]{0,10}".prop_filter_map("an artifact", |text| text.parse().ok()).boxed()
    }

    fn sense() -> BoxedStrategy<Sense> {
        (
            select(SenseKind::VARIANTS),
            select(Specificity::VARIANTS),
            select(Confidence::VARIANTS),
            select(Explicitness::VARIANTS),
        )
            .prop_map(|(kind, specificity, confidence, explicitness)| {
                Sense::new(kind, specificity, confidence, explicitness)
            })
            .boxed()
    }

    fn stage() -> BoxedStrategy<AnyStage> {
        prop_oneof![
            Just(AnyStage::from(Vague)),
            Just(AnyStage::from(Refining)),
            (Generated::artifact(), Generated::sentence::<Condition>())
                .prop_map(|(artifact, condition)| AnyStage::from(Terminal::new(artifact, condition))),
            Generated::sentence::<Reason>().prop_map(|reason| AnyStage::from(Dropped::from(reason))),
        ]
        .boxed()
    }

    fn summary() -> BoxedStrategy<Summary> {
        (
            (Generated::node(), Generated::project(), select(Facet::VARIANTS), Generated::entered()),
            (select(Role::VARIANTS), Generated::entered(), Generated::stage(), select(Basis::VARIANTS)),
            proptest::option::of(Generated::text()),
        )
            .prop_map(|((node, project, facet, entered), (role, cast, stage, basis), title)| {
                let entry = Entry::new(project, facet, entered);
                let loaded = Node::loaded(node, entry, Casting::new(role, cast), stage);
                Summary::from(
                    &title.into_iter().fold(loaded.based(basis), |node, title| node.titled(Title::new(title))),
                )
            })
            .boxed()
    }

    fn directive() -> BoxedStrategy<Directive> {
        prop_oneof![
            (Generated::segment(), Generated::node(), select(SpanRelation::VARIANTS)).prop_map(
                |(segment, node, relation)| Directive::Place { segment, node: NodeRef::Stored(node), relation }
            ),
            (Generated::segment(), Generated::sense()).prop_map(|(segment, sense)| Directive::Sense { segment, sense }),
            (Generated::node(), select(Facet::VARIANTS), "[^\r\n]{0,24}").prop_map(|(node, facet, title)| {
                Directive::Propose {
                    proposal: Proposal::Anonymous(NodeRef::Stored(node)),
                    facet,
                    title: Title::new(title),
                }
            }),
            Generated::segment().prop_map(|segment| Directive::Conversational { segment }),
        ]
        .boxed()
    }

    fn remark() -> BoxedStrategy<Remark> {
        prop_oneof![
            (Generated::segment(), select(SpanRelation::VARIANTS), Generated::node())
                .prop_map(|(segment, relation, node)| Remark::Place { segment, relation, node }),
            (Generated::segment(), Generated::sense()).prop_map(|(segment, sense)| Remark::Sense { segment, sense }),
            (Generated::segment(), select(SegmentRelation::VARIANTS), Generated::segment())
                .prop_map(|(source, relation, target)| Remark::Link { source, relation, target }),
            Generated::segment().prop_map(|segment| Remark::Conversational { segment }),
        ]
        .boxed()
    }

    fn counted() -> BoxedStrategy<Line> {
        prop_oneof![
            (Generated::text(), any::<u64>()).prop_map(|(session, cursor)| Line::Read {
                session: SessionId::new(session),
                cursor: Cursor::new(Offset::new(cursor))
            }),
            (Generated::author(), Generated::count()).prop_map(|(author, count)| Line::Authored { author, count }),
            (Generated::text(), Generated::count())
                .prop_map(|(kind, count)| Line::Unrecognised { kind: transcript_ports::Kind::new(kind), count }),
            (select(Shape::VARIANTS), Generated::count()).prop_map(|(shape, count)| Line::Formed { shape, count }),
            (select(Relation::VARIANTS), Generated::count())
                .prop_map(|(relation, count)| Line::Related { relation, count }),
            Generated::count().prop_map(Line::Written),
            (Generated::node(), Generated::project()).prop_map(|(node, project)| Line::Root { node, project }),
            (Generated::node(), Generated::project()).prop_map(|(node, project)| Line::Consumes { node, project }),
        ]
        .boxed()
    }

    fn shown_node() -> BoxedStrategy<Line> {
        prop_oneof![
            Generated::summary().prop_map(Line::Node),
            Generated::artifact().prop_map(Line::Artifact),
            Generated::sentence::<Condition>().prop_map(Line::Condition),
            Generated::sentence::<Reason>().prop_map(Line::Reason),
            (select(Kin::VARIANTS), Generated::summary()).prop_map(|(kin, node)| Line::Relative { kin, node }),
            (Generated::node(), select(NodeRelation::VARIANTS), Generated::node(), Generated::entered()).prop_map(
                |(source, relation, target, entered)| Line::Edge {
                    edge: NodeEdge::new(source, relation, target),
                    entered
                }
            ),
            (select(Role::VARIANTS), Generated::entered()).prop_map(|(role, entered)| Line::Role { role, entered }),
            (
                (select(SpanRelation::VARIANTS), Generated::segment(), Generated::author()),
                (proptest::option::of(Generated::sense()), Generated::entered(), Generated::text())
            )
                .prop_map(|((relation, segment, author), (sense, entered, body))| Line::Placed {
                    relation,
                    segment,
                    author,
                    sense,
                    entered,
                    body: Body::new(body)
                }),
        ]
        .boxed()
    }

    fn shown_message() -> BoxedStrategy<Line> {
        prop_oneof![
            (Generated::uuid(), Generated::author(), Generated::text(), Generated::text()).prop_map(
                |(uuid, author, session, stamp)| Line::Message {
                    uuid,
                    author,
                    session: SessionId::new(session),
                    stamp: Instant::new(stamp)
                }
            ),
            (Generated::segment(), Generated::shape(), Generated::text())
                .prop_map(|(segment, shape, body)| Line::Segment { segment, shape, body: Body::new(body) }),
            (Generated::anchored(), Generated::relation(), Generated::anchored())
                .prop_map(|(source, relation, target)| Line::Link { source, relation, target }),
            (Generated::remark(), select(Standing::VARIANTS), Generated::entered())
                .prop_map(|(remark, standing, entered)| Line::Annotated { remark, standing, entered }),
            (Generated::segment(), Generated::author(), Generated::text(), Generated::text()).prop_map(
                |(segment, author, stamp, body)| Line::Hit {
                    segment,
                    author,
                    stamp: Instant::new(stamp),
                    body: Body::new(body)
                }
            ),
        ]
        .boxed()
    }

    fn applied() -> BoxedStrategy<Line> {
        prop_oneof![
            Generated::directive().prop_map(Line::Applied),
            (Generated::node(), Generated::directive())
                .prop_map(|(node, directive)| Line::Proposed { node, directive }),
            Generated::segment().prop_map(Line::Uncovered),
            (0..i64::MAX, Generated::text())
                .prop_map(|(file, stamp)| Line::Repeated { file: AppliedId::new(file), stamp: Instant::new(stamp) }),
        ]
        .boxed()
    }

    fn lapse() -> BoxedStrategy<Lapse> {
        prop_oneof![
            "[^\u{0}]{0,20}".prop_map(|refusal| Lapse::refused(&std::io::Error::other(refusal))),
            (Generated::segment(), proptest::collection::vec(Generated::segment(), 0..4))
                .prop_map(|(first, rest)| Lapse::from(Uncovered::new(first, rest))),
            "[ -~]{0,20}"
                .prop_filter_map("a misread reply", |reply| reply.parse::<Directives>().err().map(Lapse::from)),
        ]
        .boxed()
    }

    fn usage() -> BoxedStrategy<Usage> {
        (any::<Tokens>(), any::<Tokens>(), any::<Tokens>(), any::<Tokens>())
            .prop_map(|(input, output, written, read)| Usage::new(input, output, written, read))
            .boxed()
    }

    fn structured() -> BoxedStrategy<Line> {
        prop_oneof![
            (0..i64::MAX, select(RunState::VARIANTS))
                .prop_map(|(run, state)| Line::Reaped { run: RunId::new(run), state }),
            (0..i64::MAX).prop_map(|job| Line::Interrupted(JobId::new(job))),
            (
                (0..i64::MAX, 0..i64::MAX, select(RunState::VARIANTS)),
                (proptest::option::of(Generated::usage()), proptest::option::of(Generated::text()))
            )
                .prop_map(|((run, job, state), (usage, reply))| Line::Ran(
                    Attempted::new(RunId::new(run), JobId::new(job), state, usage).replied(reply.map(Excerpt::new))
                )),
            (0..i64::MAX, Generated::lapse())
                .prop_map(|(job, lapse)| Line::Retried(Lapsed::new(JobId::new(job), lapse))),
            (0..i64::MAX, Generated::lapse())
                .prop_map(|(job, lapse)| Line::Flailed(Lapsed::new(JobId::new(job), lapse))),
            (Generated::segment(), 0..i64::MAX, 0..i64::MAX).prop_map(|(segment, job, run)| Line::FlailedSegment(
                Flailing::new(segment, JobId::new(job), RunId::new(run))
            )),
        ]
        .boxed()
    }

    fn viewed() -> BoxedStrategy<Line> {
        prop_oneof![
            (
                (Generated::node(), Generated::project(), select(Facet::VARIANTS), select(Stage::VARIANTS)),
                (
                    select(Basis::VARIANTS),
                    proptest::option::of(Generated::node()),
                    proptest::option::of(Generated::text())
                )
            )
                .prop_map(|((node, project, facet, stage), (basis, parent, title))| Viewed::Outlined(
                    Outline::new(node, project, facet, stage)
                        .based(basis)
                        .parented(parent)
                        .titled(title.map(Title::new))
                )),
            (
                (Generated::node(), select(SpanRelation::VARIANTS), Generated::segment()),
                (Generated::author(), Generated::entered(), Generated::text())
            )
                .prop_map(|((node, relation, segment), (author, entered, body))| Viewed::Standing {
                    node,
                    relation,
                    segment,
                    author,
                    entered,
                    body: Body::new(body)
                }),
            (Generated::segment(), Generated::text())
                .prop_map(|(segment, body)| Viewed::Unplaced { segment, body: Body::new(body) }),
        ]
        .prop_map(Line::Viewed)
        .boxed()
    }

    fn noted() -> BoxedStrategy<Line> {
        let reference = prop_oneof![
            Generated::text().prop_map(|quote| Reference::Quote(Quote::new(quote))),
            "[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}|[0-9a-f]{8}".prop_map(|cite| Reference::Cite(Cite::new(cite))),
        ];
        prop_oneof![
            (any::<usize>(), Generated::node())
                .prop_map(|(line, node)| Noted::Imported { line: NotesLine::new(line), node }),
            (any::<usize>(), reference, select(Miss::VARIANTS)).prop_map(|(line, reference, miss)| Noted::Unresolved {
                line: NotesLine::new(line),
                reference,
                miss
            }),
            any::<usize>().prop_map(|line| Noted::Unanchored(NotesLine::new(line))),
            (any::<usize>(), "[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}|[0-9a-f]{8}", Generated::count()).prop_map(
                |(line, cite, segments)| Noted::Ambiguous {
                    line: NotesLine::new(line),
                    cite: Cite::new(cite),
                    segments
                }
            ),
            (Generated::count(), Generated::count(), Generated::count(), Generated::count()).prop_map(
                |(lines, nodes, rulings, anchored)| Noted::Reconciled(Reconciliation::new(
                    lines, nodes, rulings, anchored
                ))
            ),
            (Generated::message(), Generated::author(), Generated::entered())
                .prop_map(|(message, author, entered)| Noted::Cites { message: Prefix::new(message), author, entered }),
        ]
        .prop_map(Line::Noted)
        .boxed()
    }

    fn line() -> BoxedStrategy<Line> {
        prop_oneof![
            Generated::counted(),
            Generated::shown_node(),
            Generated::shown_message(),
            Generated::applied(),
            Generated::structured(),
            Generated::viewed(),
            Generated::noted()
        ]
        .boxed()
    }
}

laws::roundtrip!(
    every_output_line_is_displayed_and_parsed_back,
    Generated::line(),
    |line: &Line| line.to_string(),
    |text: &String| text.parse::<Line>().ok(),
    Some
);

const SPREAD: [&str; 3] = [
    "12345678-0000-4000-8000-000000000001",
    "abcdef01-0000-4000-8000-000000000001",
    "abcdef01-1111-4000-8000-000000000001",
];

struct Spread;

impl Spread {
    fn applying() -> Result<Applying, Failure> {
        let session = Found::said(&[
            Said::by(Speaker::Owner, SPREAD[0], "first thought. second thought".to_owned()),
            Said::by(Speaker::Owner, SPREAD[1], "third thought".to_owned()),
            Said::by(Speaker::Owner, SPREAD[2], "fourth thought".to_owned()),
        ]);
        let applying = Applying { store: Sqlite::in_memory::<StreamSchema>(), files: session.files() };
        applying.run(&["ingest"])?;
        applying.run(&["init", "demo"])?;
        Ok(applying)
    }

    fn parsed(out: &str) -> Vec<Line> {
        let ends: Vec<usize> = out.match_indices('\n').map(|(end, _)| end).collect();
        let starts = [0].into_iter().chain(ends.iter().map(|end| end.saturating_add(1)));
        starts
            .zip(ends.iter())
            .filter_map(|(start, end)| out.get(start..*end))
            .filter_map(|line| line.parse::<Line>().ok())
            .collect()
    }

    fn reference(prefix: &str, ordinal: usize) -> SegmentRef {
        SegmentRef::new(Prefix::new(prefix.to_owned()), Ordinal::new(ordinal))
    }

    fn segment(prefix: &str, ordinal: usize, body: &str) -> Line {
        Line::Segment {
            segment: Spread::reference(prefix, ordinal),
            shape: ShapeName::new("sentence".to_owned()),
            body: Body::new(body.to_owned()),
        }
    }

    fn message(uuid: &str) -> Line {
        Line::Message {
            uuid: Uuid::new(uuid.to_owned()),
            author: Authorship::Owner,
            session: SessionId::new("s".to_owned()),
            stamp: Instant::new("2026-01-01T00:00:00Z".to_owned()),
        }
    }
}

#[test]
fn show_and_search_print_the_shortest_unique_ref_and_every_one_printed_applies() -> Result<(), Failure> {
    let applying = Spread::applying()?;
    let shown = [
        Spread::parsed(&applying.run(&["show", SPREAD[0]])?),
        Spread::parsed(&applying.run(&["show", SPREAD[1]])?),
        Spread::parsed(&applying.run(&["show", SPREAD[2]])?),
    ];
    let wanted = [
        vec![
            Spread::message(SPREAD[0]),
            Spread::segment("12345678", 0, "first thought."),
            Spread::segment("12345678", 1, "second thought"),
        ],
        vec![Spread::message(SPREAD[1]), Spread::segment("abcdef01-0", 0, "third thought")],
        vec![Spread::message(SPREAD[2]), Spread::segment("abcdef01-1", 0, "fourth thought")],
    ];
    assert_eq!(shown, wanted);
    let found = Spread::parsed(&applying.run(&["search", "thought"])?);
    let hit = |prefix: &str, ordinal: usize, body: &str| Line::Hit {
        segment: Spread::reference(prefix, ordinal),
        author: Authorship::Owner,
        stamp: Instant::new("2026-01-01T00:00:00Z".to_owned()),
        body: Body::new(body.to_owned()),
    };
    let hits = [
        hit("12345678", 0, "first thought."),
        hit("12345678", 1, "second thought"),
        hit("abcdef01-0", 0, "third thought"),
        hit("abcdef01-1", 0, "fourth thought"),
    ];
    assert_eq!(found.len(), hits.len());
    assert!(hits.iter().all(|hit| found.contains(hit)));
    let refs = ["12345678.0", "12345678.1", "abcdef01-0.0", "abcdef01-1.0"];
    let directives: Vec<String> = refs.iter().map(|reference| format!("conversational({reference})")).collect();
    let applied = Spread::parsed(&applying.applied(&directives.join("\n"))?);
    let echoed: Vec<Line> =
        directives.iter().filter_map(|directive| directive.parse().ok()).map(Line::Applied).collect();
    assert_eq!(echoed.len(), refs.len());
    assert!(echoed.iter().all(|echo| applied.contains(echo)));
    refs.iter().try_for_each(|reference| applying.run(&["show", reference]).map(|_shown| ()))
}

#[test]
fn show_takes_the_message_and_segment_prefixes_apply_takes_and_refuses_an_ambiguous_or_unknown_one()
-> Result<(), Failure> {
    let applying = Spread::applying()?;
    let whole = Spread::parsed(&applying.run(&["show", SPREAD[0]])?);
    assert_eq!(Spread::parsed(&applying.run(&["show", "12345678"])?), whole);
    assert_eq!(Spread::parsed(&applying.run(&["show", "12345678.1"])?), whole);
    assert!(matches!(applying.run(&["show", "abcdef01"]), Err(Failure::Unmatched(Unmatched::Ambiguous(_)))));
    assert!(matches!(applying.run(&["show", "abcdef01.0"]), Err(Failure::Unmatched(Unmatched::Ambiguous(_)))));
    assert!(matches!(applying.run(&["show", "fedcba98"]), Err(Failure::Unmatched(Unmatched::NoMatch(_)))));
    assert!(matches!(applying.run(&["show", "12345678.7"]), Err(Failure::Unresolved(Unresolved::Unsegmented(_)))));
    let applied = applying.applied("conversational(12345678.1)");
    assert!(applied.is_ok(), "{applied:?}");
    assert!(matches!(
        applying.applied("conversational(abcdef01.0)"),
        Err(Failure::Unresolved(Unresolved::Ambiguous(_)))
    ));
    assert!(matches!(applying.applied("conversational(fedcba98.0)"), Err(Failure::Unresolved(Unresolved::Unheard(_)))));
    Ok(())
}

struct Ends;

impl Ends {
    fn of(line: Line) -> Vec<Anchored> {
        match line {
            Line::Link { source, target, .. } => vec![source, target],
            _ => Vec::new(),
        }
    }

    fn conversational(anchor: &Anchored) -> Option<String> {
        match anchor {
            Anchored::Segment(segment) => Some(format!("conversational({segment})")),
            Anchored::Message(_) => None,
        }
    }
}

#[test]
fn review_every_segment_ref_on_a_shown_link_line_is_shown_again() -> Result<(), Failure> {
    let applying = Applying::ready()?;
    let shown = applying.run(&["show", "aaaaaaaa-0000-4000-8000-000000000003"])?;
    let anchors: Vec<Anchored> = Spread::parsed(&shown).into_iter().flat_map(Ends::of).collect();
    assert_ne!(anchors, Vec::<Anchored>::new(), "{shown}");
    let refused: Vec<&Anchored> =
        anchors.iter().filter(|anchor| applying.run(&["show", &anchor.to_string()]).is_err()).collect();
    assert_eq!(refused, Vec::<&Anchored>::new(), "{shown}");
    let directives: Vec<String> = anchors.iter().filter_map(Ends::conversational).collect();
    assert_ne!(directives, Vec::<String>::new(), "{shown}");
    let applied = applying.applied(&directives.join("\n"));
    assert!(applied.is_ok(), "{applied:?}");
    Ok(())
}

#[test]
fn review_a_seven_character_prefix_is_shown_whatever_its_first_character() -> Result<(), Failure> {
    let session = Found::said(&[
        Said::by(Speaker::Owner, "1a2b3c4d-0000-4000-8000-000000000001", "first thought".to_owned()),
        Said::by(Speaker::Owner, "a1b2c3d4-0000-4000-8000-000000000001", "second thought".to_owned()),
    ]);
    let applying = Applying { store: Sqlite::in_memory::<StreamSchema>(), files: session.files() };
    applying.run(&["ingest"])?;
    let lettered = applying.run(&["show", "a1b2c3d"]).map(|_shown| ());
    let numbered = applying.run(&["show", "1a2b3c4"]).map(|_shown| ());
    assert!(lettered.is_ok(), "{lettered:?}");
    assert!(numbered.is_ok(), "{numbered:?}");
    Ok(())
}

const STEMS: [&str; 3] = ["aaaa1111", "aaaa1112", "bbbb2222"];

const SAYINGS: [&str; 3] = ["", "word", "1. word\\n2. more word"];

#[derive(Clone, Debug)]
struct Uttered {
    stem: &'static str,
    speaker: Speaker,
    text: &'static str,
    back: Option<usize>,
}

struct Echoes;

impl Echoes {
    fn uttered() -> BoxedStrategy<Vec<Uttered>> {
        proptest::collection::vec(
            (
                select(STEMS.to_vec()),
                select(vec![Speaker::Owner, Speaker::Lead]),
                select(SAYINGS.to_vec()),
                proptest::option::of(0usize..3),
            )
                .prop_map(|(stem, speaker, text, back)| Uttered { stem, speaker, text, back }),
            1..6,
        )
        .boxed()
    }

    fn uuid(stem: &str, index: usize) -> String {
        format!("{stem}-0000-4000-8000-{index:012}")
    }

    fn uuids(uttered: &[Uttered]) -> Vec<String> {
        uttered.iter().enumerate().map(|(index, said)| Echoes::uuid(said.stem, index)).collect()
    }

    fn session(uttered: &[Uttered]) -> Session {
        let uuids = Echoes::uuids(uttered);
        Session::working(
            uttered
                .iter()
                .zip(&uuids)
                .enumerate()
                .map(|(index, (said, uuid))| {
                    let parent = said
                        .back
                        .and_then(|back| index.checked_sub(back.saturating_add(1)))
                        .and_then(|earlier| uuids.get(earlier))
                        .map_or_else(|| "null".to_owned(), |earlier| format!("\"{earlier}\""));
                    Session::line(said.speaker, uuid, &parent, said.text)
                })
                .collect(),
        )
    }

    fn refs(line: Line) -> Vec<String> {
        match line {
            Line::Message { uuid, .. } => vec![uuid.to_string()],
            Line::Segment { segment, .. } | Line::Hit { segment, .. } => vec![segment.to_string()],
            Line::Link { source, target, .. } => vec![source.to_string(), target.to_string()],
            _ => Vec::new(),
        }
    }
}

laws::law!(every_ref_that_show_or_search_prints_is_shown, Echoes::uttered(), uttered, {
    let applying = Applying { store: Sqlite::in_memory::<StreamSchema>(), files: Echoes::session(&uttered).files() };
    applying.run(&["ingest"]).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let printed = Echoes::uuids(&uttered)
        .iter()
        .map(|uuid| applying.run(&["show", uuid]))
        .chain([applying.run(&["search", "--all", "word"])])
        .collect::<Result<String, Failure>>()
        .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let refs: Vec<String> = Spread::parsed(&printed).into_iter().flat_map(Echoes::refs).collect();
    prop_assert!(refs.len() >= uttered.len(), "{}", printed);
    let refused: Vec<&String> = refs.iter().filter(|reference| applying.run(&["show", reference]).is_err()).collect();
    prop_assert_eq!(refused, Vec::<&String>::new(), "{}", printed);
});
