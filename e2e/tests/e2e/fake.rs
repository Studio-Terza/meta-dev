use std::io;
use std::path::{Path, PathBuf};

use ci::ports::{Invocation, Ports, Run, Text};
use file_ports::Memory;

#[derive(Debug, Clone, Copy)]
pub enum Edit {
    Modified,
    Added,
}

impl Edit {
    fn status(self) -> &'static str {
        match self {
            Edit::Modified => "M",
            Edit::Added => "A",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    edit: Edit,
    path: PathBuf,
}

impl Change {
    pub fn new(edit: Edit, path: PathBuf) -> Change {
        Change { edit, path }
    }
}

#[derive(Debug)]
pub struct Fake {
    metadata: String,
    changes: Vec<Change>,
    files: Memory,
}

impl Fake {
    const FORK: &'static str = "fork\n";
    const UNLABELLED_CHANGE: &'static str = "change\n";

    pub fn new(metadata: &str, changes: Vec<Change>) -> Fake {
        Fake { metadata: metadata.to_owned(), changes, files: Memory::default() }
    }

    pub fn modifying(metadata: &str, changed: &[PathBuf]) -> Fake {
        Fake::new(metadata, changed.iter().map(|path| Change::new(Edit::Modified, path.clone())).collect())
    }

    fn names(&self) -> String {
        self.changes.iter().map(|change| format!("{}\n", change.path.display())).collect()
    }

    fn statuses(&self) -> String {
        self.changes.iter().map(|change| format!("{}\t{}\n", change.edit.status(), change.path.display())).collect()
    }

    fn git(&self, invocation: &Invocation) -> String {
        let asks = |word: &str| invocation.arguments().iter().any(|argument| argument == word);
        if asks("merge-base") {
            Self::FORK.to_owned()
        } else if asks("--name-status") {
            self.statuses()
        } else {
            self.names()
        }
    }

    fn refused(invocation: &Invocation) -> io::Error {
        io::Error::other(format!("a scenario starts no process: {}", invocation.program().display()))
    }
}

impl Ports for Fake {
    type Files = Memory;

    fn files(&self) -> &Memory {
        &self.files
    }

    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        Err(Self::refused(invocation))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<Text> {
        match invocation.program().to_str() {
            Some("cargo") => Ok(self.metadata.clone()),
            Some("git") => Ok(self.git(invocation)),
            Some("gh") => Ok(Self::UNLABELLED_CHANGE.to_owned()),
            _ => Err(Self::refused(invocation)),
        }
        .map(Text::new)
    }

    fn logged(&self, invocation: &Invocation, _log: &Path) -> io::Result<Run> {
        Err(Self::refused(invocation))
    }

    fn runs(&self, invocations: &[Invocation]) -> Vec<io::Result<Run>> {
        invocations.iter().map(|invocation| self.run(invocation)).collect()
    }

    fn outputs(&self, invocations: &[Invocation]) -> Vec<io::Result<Text>> {
        invocations.iter().map(|invocation| self.output(invocation)).collect()
    }
}
