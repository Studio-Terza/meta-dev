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
    const fn status(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    edit: Edit,
    path: PathBuf,
}

impl Change {
    pub const fn new(edit: Edit, path: PathBuf) -> Self {
        Self { edit, path }
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

    pub fn new(metadata: &str, changes: Vec<Change>) -> Self {
        Self { metadata: metadata.to_owned(), changes, files: Memory::default() }
    }

    pub fn modifying(metadata: &str, changed: &[PathBuf]) -> Self {
        Self::new(metadata, changed.iter().map(|path| Change::new(Edit::Modified, path.clone())).collect())
    }

    fn names(&self) -> String {
        self.changes.iter().map(|change| format!("{}\n", change.path.display())).collect()
    }

    fn statuses(&self) -> String {
        self.changes.iter().map(|change| format!("{}\t{}\n", change.edit.status(), change.path.display())).collect()
    }

    fn git(&self, invocation: &Invocation) -> String {
        match Asked::of(invocation) {
            Asked::MergeBase => Self::FORK.to_owned(),
            Asked::NameStatus => self.statuses(),
            Asked::Names => self.names(),
        }
    }

    fn answered(&self, program: Program, invocation: &Invocation) -> io::Result<String> {
        match program {
            Program::Cargo => Ok(self.metadata.clone()),
            Program::Git => Ok(self.git(invocation)),
            Program::Gh => Ok(Self::UNLABELLED_CHANGE.to_owned()),
            Program::Other => Err(Self::refused(invocation)),
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
        self.answered(Program::of(invocation), invocation).map(Text::new)
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

#[derive(Clone, Copy, Debug)]
enum Asked {
    MergeBase,
    NameStatus,
    Names,
}

impl Asked {
    const NAMED: [(&'static str, Self); 2] = [("merge-base", Self::MergeBase), ("--name-status", Self::NameStatus)];

    fn of(invocation: &Invocation) -> Self {
        Self::NAMED
            .iter()
            .find(|(word, _)| invocation.arguments().iter().any(|argument| argument == word))
            .map_or(Self::Names, |(_, asked)| *asked)
    }
}

#[derive(Clone, Copy, Debug)]
enum Program {
    Cargo,
    Git,
    Gh,
    Other,
}

impl Program {
    const NAMED: [(&'static str, Self); 3] = [("cargo", Self::Cargo), ("git", Self::Git), ("gh", Self::Gh)];

    fn of(invocation: &Invocation) -> Self {
        let program = invocation.program().to_str();
        Self::NAMED.iter().find(|(name, _)| program == Some(*name)).map_or(Self::Other, |(_, named)| *named)
    }
}
