use std::ffi::OsString;
use std::path::{Path, PathBuf};

use console_ports::Variables;
use file_ports::{Bytes, Files, Memory};
use newtype::Counter;
use workload::Scale;

const SESSIONS_PER_SCALE: usize = 3;
const MESSAGES_PER_SESSION: usize = 4;
const PROJECTS: &str = "/home/tester/.claude/projects/-work";

pub struct Fixture {
    pub memory: Memory,
    pub environment: Variables,
    pub sessions: usize,
}

impl Fixture {
    pub fn sessions_at(scale: Scale) -> usize {
        SESSIONS_PER_SCALE.saturating_mul(scale.count())
    }

    pub fn at(scale: Scale) -> Fixture {
        let sessions = Self::sessions_at(scale);
        let memory = Memory::default();
        memory.create_directories(Path::new(PROJECTS)).ok();
        (0..sessions).for_each(|session| {
            memory
                .write(&PathBuf::from(PROJECTS).join(format!("s{session}.jsonl")), Bytes::from(&Self::text(session)))
                .ok();
        });
        let environment = [("HOME", "/home/tester"), ("PWD", "/work")]
            .into_iter()
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .collect();
        Fixture { memory, environment, sessions }
    }

    pub fn last_message(&self) -> String {
        format!("s{}-m{}", self.sessions.saturating_sub(1), MESSAGES_PER_SESSION.saturating_sub(1))
    }

    fn text(session: usize) -> String {
        (0..MESSAGES_PER_SESSION).map(|message| Self::line(session, message)).collect()
    }

    fn line(session: usize, message: usize) -> String {
        let parent =
            message.checked_sub(1).map_or_else(|| "null".to_owned(), |before| format!("\"s{session}-m{before}\""));
        let Speaker { kind, origin } = Self::speaker(message);
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"s{session}-m{message}\",\"parentUuid\":{parent},\"sessionId\":\"s{session}\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"content\":\"1. first point. second point?\"}}}}\n"
        )
    }

    fn speaker(message: usize) -> Speaker {
        Some(message % 2).filter(|parity| *parity == 0).map_or(Speaker { kind: "assistant", origin: "null" }, |_| {
            Speaker { kind: "user", origin: "{\"kind\":\"human\"}" }
        })
    }
}

struct Speaker {
    kind: &'static str,
    origin: &'static str,
}
