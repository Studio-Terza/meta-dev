use std::ffi::OsString;
use std::path::Path;

use console_ports::Variables;
use design::{Cli, Failure, Outcome};
use file_ports::{Bytes, Files, Memory};
use port_core::Run;

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

    fn files(&self) -> Memory {
        let memory = Memory::default();
        memory.create_directories(Path::new(self.directory)).ok();
        memory.write(&Path::new(self.directory).join("s.jsonl"), Bytes::from(&self.lines)).ok();
        memory
    }
}

struct Invocation;

impl Invocation {
    fn output(session: &Session, arguments: &[&str]) -> Result<String, Failure> {
        let environment: Variables = [("HOME", HOME), ("PWD", "/work")]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect();
        let run = Run::root();
        let mut out = Vec::new();
        Cli::new(&environment, session.files()).report(&run, arguments.iter().map(OsString::from), &mut out)?;
        Ok(String::from_utf8(out).unwrap_or_default())
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

struct Answered {
    outcome: Outcome,
    out: String,
}

struct Rooted;

impl Rooted {
    fn outcome(session: &Session, arguments: &[&str]) -> Result<Answered, Failure> {
        let environment: Variables = [("HOME", HOME), ("PWD", "/work")]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect();
        let mut out = Vec::new();
        let outcome = Cli::new(&environment, session.files()).rooted(arguments.iter().map(OsString::from), &mut out)?;
        Ok(Answered { outcome, out: String::from_utf8(out).unwrap_or_default() })
    }
}

#[test]
fn a_rooted_run_writes_its_report_to_out_and_names_its_outcome() -> Result<(), Failure> {
    let ingested = Rooted::outcome(&Session::typical(), &["ingest"])?;
    assert_eq!(ingested.outcome, Outcome::Ingested);
    assert!(ingested.out.contains("authorship owner 2\n"));
    let shown = Rooted::outcome(&Session::answered(), &["show", "o1"])?;
    assert_eq!(shown.outcome, Outcome::Shown);
    Ok(())
}

#[test]
fn show_lists_the_links_that_touch_the_message() -> Result<(), Failure> {
    let out = Invocation::output(&Session::answered(), &["show", "o1"])?;
    assert!(out.contains("link l1 replies-to o1"));
    let unrelated = Invocation::output(&Session::working(Session::typical().lines), &["show", "o2"])?;
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
fn show_prints_a_message_by_uuid_from_the_live_transcript() -> Result<(), Failure> {
    let out = Invocation::output(&Session::typical(), &["show", "o1"])?;
    assert!(out.contains("o1 owner session s at 2026-01-01T00:00:00Z"));
    assert!(out.contains("0 item"));
    Ok(())
}

#[test]
fn show_of_an_unknown_uuid_fails_naming_it() {
    let failed = Invocation::output(&Session::typical(), &["show", "nobody"]);
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
