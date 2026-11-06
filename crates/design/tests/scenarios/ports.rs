use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Output;

use e2e::ports::Home;
use file_ports::Bytes;

#[derive(Debug)]
pub struct Design {
    home: Home,
}

impl Design {
    const SESSION: &'static str = concat!(
        "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
        "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"go\"}}\n"
    );

    pub fn with_a_session_in_work() -> io::Result<Design> {
        Home::holding(Path::new(".claude/projects/-work/s.jsonl"), &Bytes::from(Self::SESSION))
            .map(|home| Design { home })
    }

    pub fn run(&self, arguments: &[&str]) -> io::Result<Output> {
        let words: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        self.home.run(Path::new(env!("CARGO_BIN_EXE_design")), &words)
    }
}
