use std::ffi::OsString;
use std::path::Path;

use console_ports::Variables;
use design::{Cli, Failure};
use file_ports::{Bytes, Files, Memory};
use port_core::{Rooting, Run};

const HOME: &str = "/home/tester";

#[derive(Clone, Copy)]
enum Speaker {
    Owner,
    Lead,
    Bystander,
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
            Speaker::Bystander => Wire { kind: "user", origin: "null" },
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

    fn line(speaker: Speaker, uuid: &str, parent: &str, content: &str) -> String {
        let Wire { kind, origin } = speaker.wire();
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":{parent},\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"content\":\"{content}\"}}}}\n"
        )
    }

    fn typical() -> Session {
        Session::working(
            [
                Self::line(Speaker::Owner, "o1", "null", "1. go. now"),
                Self::line(Speaker::Lead, "l1", "null", "done"),
                Self::line(Speaker::Owner, "o2", "null", "thanks"),
                Self::line(Speaker::Bystander, "b1", "null", "result"),
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
        let run = Run::root(Rooting::Shared)?;
        let mut out = Vec::new();
        Cli::new(&environment, session.files()).report(&run, arguments.iter().map(OsString::from), &mut out)?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

#[test]
fn ingest_counts_each_authorship_form_and_relation() -> Result<(), Failure> {
    let out = Invocation::output(&Session::typical(), &["ingest"])?;
    assert!(out.contains("authorship owner 2\n"));
    assert!(out.contains("authorship lead 1\n"));
    assert!(out.contains("authorship bystander 1\n"));
    assert!(out.contains("form item 1\n"));
    assert!(out.contains("form sentence 3\n"));
    Ok(())
}

#[test]
fn ingest_counts_a_reply_once() -> Result<(), Failure> {
    let lines = [
        Session::line(Speaker::Owner, "o1", "null", "question"),
        Session::line(Speaker::Lead, "l1", "\"o1\"", "answer"),
    ]
    .concat();
    let out = Invocation::output(&Session::working(lines), &["ingest"])?;
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
    let out = Invocation::output(&session, &["ingest", "--workspace", "/other"])?;
    assert!(out.contains("authorship owner 2\n"));
    Ok(())
}

#[test]
fn an_unknown_command_is_a_usage_failure() {
    assert!(matches!(Invocation::output(&Session::typical(), &["frobnicate"]), Err(Failure::Usage)));
}
