use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Output;

use e2e::ports::Home;
use file_ports::Bytes;
use grammar::slug::Letters;

#[derive(Debug)]
pub struct Design {
    home: Home,
    workspace: PathBuf,
}

impl Design {
    const SESSION: &'static str = concat!(
        "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
        "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"go\"}}\n"
    );

    const AGENT: &'static str = concat!(
        "#!/bin/sh\ncat >/dev/null\necho '{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,",
        "\"result\":\"place(\",\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}'\n"
    );

    const NINE: &'static str = concat!(
        "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
        "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},",
        "\"message\":{\"content\":\"1. a\\n2. b\\n3. c\\n4. d\\n5. e\\n6. f\\n7. g\\n8. h\\n9. i\"}}\n"
    );

    const DYING: &'static str = concat!(
        "#!/bin/sh\ncat >/dev/null\ncalls=$(cat \"$HOME/calls\" 2>/dev/null)\ncalls=$((calls + 1))\n",
        "echo \"$calls\" >\"$HOME/calls\"\n[ \"$calls\" -eq 3 ] && kill -9 \"$PPID\"\n",
        "echo '{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,",
        "\"result\":\"place(\",\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}'\n"
    );

    pub fn with_a_session_in_work() -> io::Result<Design> {
        Design::saying(Self::SESSION, Self::AGENT)
    }

    pub fn with_two_jobs_whose_agent_dies_on_the_second() -> io::Result<Design> {
        Design::saying(Self::NINE, Self::DYING)
    }

    fn saying(said: &str, agent: &str) -> io::Result<Design> {
        let fresh = Home::new()?;
        let workspace = AsRef::<Path>::as_ref(&fresh).join("work");
        let slug = Letters::slugged(&workspace.to_string_lossy());
        let session = Path::new(".claude/projects").join(slug).join("s.jsonl");
        let program = Path::new("bin/claude");
        let home = fresh.holding(&session, &Bytes::from(said))?.holding(program, &Bytes::from(agent))?;
        Ok(Design { home, workspace })
    }

    pub fn run(&self, arguments: &[&str]) -> io::Result<Output> {
        let words: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        self.home.run(Path::new(env!("CARGO_BIN_EXE_design")), &words, &Bytes::default())
    }

    pub fn in_work(&self, arguments: &[&str], input: &str) -> io::Result<Output> {
        let words: Vec<OsString> = arguments
            .iter()
            .map(OsString::from)
            .chain([OsString::from("--workspace"), self.workspace.clone().into_os_string()])
            .collect();
        self.home.run(Path::new(env!("CARGO_BIN_EXE_design")), &words, &Bytes::from(input))
    }
}
