use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use port_core::Run;
use transcript_ports::{ClaudeCodeDir, Cursor, Line, Transcripts};

struct Torn(&'static str);

impl Torn {
    const GOOD: &'static str = "{\"type\":\"user\",\"uuid\":\"1\",\"parentUuid\":null,\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:01Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"typed words\"}}\n";

    fn kinds(&self) -> Option<Vec<String>> {
        let memory = Memory::default();
        memory.create_directories(Path::new("/p")).ok()?;
        memory.append(Path::new("/p/s1.jsonl"), &Bytes::from(&[Self::GOOD, self.0, Self::GOOD].concat())).ok()?;
        let run = Run::root();
        let read = {
            let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from("/p")));
            let sessions = Transcripts::sessions(&port).ok()?;
            Transcripts::read_from(&port, sessions.first()?, Cursor::default()).ok()?
        };
        run.finish().settled().ok()?;
        Some(read.into_iter().map(|line| Torn::kind(&line)).collect())
    }

    fn kind(line: &Line) -> String {
        match line {
            Line::Said(_) => String::from("said"),
            Line::Skipped(_) => String::from("skipped"),
            Line::Unrecognised(unrecognised) => unrecognised.kind().to_string(),
        }
    }
}

laws::table!(
    a_line_that_is_not_a_record_is_read_as_unreadable_and_the_rest_still_reads,
    Torn::kinds,
    [
        Torn("{\"type\":\"user\",\"uuid\":\"2\",\"mess\n") => Some(vec![String::from("said"), String::from("unreadable"), String::from("said")]),
        Torn("{\"foo\":1}\n") => Some(vec![String::from("said"), String::from("unreadable"), String::from("said")]),
        Torn("\n") => Some(vec![String::from("said"), String::from("said")]),
    ]
);
