use std::io;
use std::path::{Path, PathBuf};

use ci::ports::{Invocation, Ports, Run, Text};
use file_ports::Memory;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(Display, Debug, Clone, Copy)]
pub enum Edit {
    #[strum(serialize = "M")]
    Modified,
    #[strum(serialize = "A")]
    Added,
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
        self.changes.iter().map(|change| format!("{}\t{}\n", change.edit, change.path.display())).collect()
    }

    fn git(&self, invocation: &Invocation) -> String {
        match Asked::of(invocation) {
            Some(Asked::MergeBase) => Self::FORK.to_owned(),
            Some(Asked::NameStatus) => self.statuses(),
            None => self.names(),
        }
    }

    fn answered(&self, program: Option<Program>, invocation: &Invocation) -> io::Result<String> {
        match program {
            Some(Program::Cargo) => Ok(self.metadata.clone()),
            Some(Program::Git) => Ok(self.git(invocation)),
            Some(Program::Gh) => Ok(Self::UNLABELLED_CHANGE.to_owned()),
            None => Err(Self::refused(invocation)),
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

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug)]
enum Asked {
    #[strum(serialize = "merge-base")]
    MergeBase,
    #[strum(serialize = "--name-status")]
    NameStatus,
}

impl Asked {
    fn of(invocation: &Invocation) -> Option<Asked> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|asked| invocation.arguments().iter().any(|argument| argument == <&'static str>::from(*asked)))
    }
}

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
enum Program {
    Cargo,
    Git,
    Gh,
}

impl Program {
    fn of(invocation: &Invocation) -> Option<Program> {
        let program = invocation.program().to_str();
        Self::VARIANTS.iter().copied().find(|named| program == Some(<&'static str>::from(*named)))
    }
}
