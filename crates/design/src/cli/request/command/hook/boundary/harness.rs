mod boundary;

use std::path::{Path, PathBuf};

use file_ports::{Batched, Epoch, Files, Kind, Tail};
use invariants::Ignoring;

use crate::cli::request::ignored::Ignored;
pub use boundary::Tasks;
use boundary::{Output, Task};

#[derive(Debug)]
pub struct Harness<'files, F: Files> {
    files: &'files F,
    now: Epoch,
}

impl<'files, F: Files> Harness<'files, F> {
    const TEMPORARY: &'static str = "/private/tmp";
    const TASKS: &'static str = "tasks";
    const SUBAGENTS: &'static str = "subagents";

    pub fn new(files: &'files F, now: Epoch) -> Harness<'files, F> {
        Harness { files, now }
    }

    pub fn tasks(&self, transcript: &Path) -> Tasks {
        let listed: Vec<PathBuf> = self
            .files
            .entries_many(&self.folders(transcript))
            .into_iter()
            .flat_map(|entries| entries.ignoring(Ignored::Untasked).unwrap_or_default())
            .collect();
        let outputs: Vec<Output> = listed.iter().filter_map(|path| Output::listed(path)).collect();
        let sizes = self.files.sizes(&outputs);
        let tailed: Vec<Option<Tail>> = sizes
            .into_iter()
            .zip(&outputs)
            .map(|(size, output)| output.tailed(size.ignoring(Ignored::Untasked)?))
            .collect();
        let reads: Vec<Tail> = tailed.iter().flatten().cloned().collect();
        let tails = self.files.read_tails(&reads);
        let stamps = self.files.stamps_many(&reads);
        outputs
            .into_iter()
            .zip(tailed)
            .filter_map(|(output, tail)| tail.map(|_read| output))
            .zip(tails)
            .zip(stamps)
            .filter_map(|((output, tail), stamped)| {
                let read = tail.ignoring(Ignored::Untasked)?;
                Task::new(output, &read, stamped.ignoring(Ignored::Untasked)?, self.now)
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
        let candidates: Vec<PathBuf> = tasks.into_iter().chain([subagents]).collect();
        let kinds = self.files.kinds(&candidates);
        candidates
            .into_iter()
            .zip(kinds)
            .filter(|(_folder, kind)| *kind == Kind::Directory)
            .map(|(folder, _kind)| folder)
            .collect()
    }
}
