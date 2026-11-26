use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use invariants::{Ignoring, ignored};
use port_core::Run;
use transcript_ports::{ClaudeCodeDir, Cursor, Line, Transcripts};

ignored! {
    enum Ignored {
        Unread => "a seed, read or settle that fails yields no kinds, which the case then reports",
    }
}

struct Torn(&'static str);

impl Torn {
    const GOOD: &'static str = "{\"type\":\"user\",\"uuid\":\"1\",\"parentUuid\":null,\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:01Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"typed words\"}}\n";

    fn kinds(&self) -> Option<Vec<String>> {
        let memory = Memory::default();
        memory.create_directories(Path::new("/p")).ignoring(Ignored::Unread)?;
        memory
            .append(Path::new("/p/s1.jsonl"), &Bytes::from(&[Self::GOOD, self.0, Self::GOOD].concat()))
            .ignoring(Ignored::Unread)?;
        let run = Run::root();
        let read = {
            let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from("/p")));
            let sessions = Transcripts::sessions(&port).ignoring(Ignored::Unread)?;
            Transcripts::read_from(&port, sessions.first()?, Cursor::default()).ignoring(Ignored::Unread)?
        };
        run.finish().settled().ignoring(Ignored::Unread)?;
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

#[test]
fn a_line_that_is_not_a_record_is_read_as_unreadable_and_the_rest_still_reads() {
    let torn = ["said", "unreadable", "said"].map(String::from).to_vec();
    assert_eq!(Torn("{\"type\":\"user\",\"uuid\":\"2\",\"mess\n").kinds(), Some(torn.clone()));
    assert_eq!(Torn("{\"foo\":1}\n").kinds(), Some(torn));
    assert_eq!(Torn("\n").kinds(), Some(["said", "said"].map(String::from).to_vec()));
}
