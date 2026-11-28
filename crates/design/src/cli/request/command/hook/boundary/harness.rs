mod boundary;

use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use invariants::Ignoring;
use timer_ports::Wall;

use crate::cli::request::ignored::Ignored;
pub use boundary::{Halt, Stopped, Tasks};
use boundary::{Output, Task};

#[derive(Debug)]
pub struct Harness<'files, F: Files> {
    files: &'files F,
    now: Wall,
}

impl<'files, F: Files> Harness<'files, F> {
    const TEMPORARY: &'static str = "/private/tmp";
    const TASKS: &'static str = "tasks";
    const SUBAGENTS: &'static str = "subagents";

    pub fn new(files: &'files F, now: Wall) -> Harness<'files, F> {
        Harness { files, now }
    }

    pub fn tasks(&self, transcript: &Path) -> Tasks {
        let listed: Vec<PathBuf> = self
            .folders(transcript)
            .iter()
            .flat_map(|folder| self.files.entries(folder).ignoring(Ignored::Untasked).unwrap_or_default())
            .collect();
        listed
            .iter()
            .filter_map(|path| Output::listed(path))
            .filter_map(|output| {
                let tail = output.tailed(self.files.size(output.as_ref()).ignoring(Ignored::Untasked)?)?;
                let read = self.files.read_from(tail.as_ref(), tail.offset()).ignoring(Ignored::Untasked)?;
                let stamped = self.files.stamps(tail.as_ref()).ignoring(Ignored::Untasked)?;
                Task::new(output, &read, stamped, self.now)
            })
            .collect()
    }

    fn folders(&self, transcript: &Path) -> Vec<PathBuf> {
        let stem = transcript.file_stem();
        let slug = transcript.parent().and_then(Path::file_name);
        let roots = self.files.entries(Path::new(Self::TEMPORARY)).ignoring(Ignored::Untasked).unwrap_or_default();
        let tasks: Vec<PathBuf> = slug.zip(stem).map_or_default(|(project, session)| {
            roots.iter().map(|root| root.join(project).join(session).join(Self::TASKS)).collect()
        });
        let subagents = transcript.with_extension("").join(Self::SUBAGENTS);
        tasks.into_iter().chain([subagents]).filter(|folder| self.files.kind(folder) == Kind::Directory).collect()
    }
}
