use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;

use std::path::PathBuf;
use std::sync::Mutex;

use console_ports::Variables;
use design::{Cli, Failure, Outcome};
use design_store::{ShownMessage, Stored, StreamSchema};
use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::test_runner::TestCaseError;
use port_core::{Held, Pass, Release, ReleaseReceipt, Released, Resource, Run, port};
use store_ports::{Busy, Sqlite, Store};
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

struct Invocation;

impl Invocation {
    fn environment() -> Variables {
        [("HOME", HOME), ("PWD", "/work")]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect()
    }

    fn output(session: &Session, arguments: &[&str]) -> Result<String, Failure> {
        Self::against(&Sqlite::in_memory::<StreamSchema>(), session, arguments)
    }

    fn against(store: &Sqlite, session: &Session, arguments: &[&str]) -> Result<String, Failure> {
        let run = Run::root();
        let mut out = Vec::new();
        Cli::new(&Self::environment(), session.files(), |_path| store.clone()).report(
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

#[test]
fn the_store_lives_in_the_workspace_meta_directory() -> Result<(), Failure> {
    let opened = Mutex::new(Vec::new());
    let run = Run::root();
    Cli::new(&Invocation::environment(), Session::typical().files(), |path: PathBuf| {
        opened.lock().map(|mut paths| paths.push(path)).ok();
        Sqlite::in_memory::<StreamSchema>()
    })
    .report(&run, ["ingest"].iter().map(OsString::from), &mut Vec::new())?;
    assert_eq!(opened.into_inner().unwrap_or_default(), [PathBuf::from("/work/.meta/design.sqlite")]);
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
