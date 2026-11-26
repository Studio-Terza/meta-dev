use std::path::Path;

use design_model::Project;
use file_ports::{Bytes, FileFailure, Files, Offset};
use local_config::Kept;
use port_core::{Port, Run};
use store_ports::Store;
use transcript_ports::ClaudeCodeDir;

use super::projects::Located;
use crate::cli::failure::Failure;

#[derive(Debug)]
pub struct Ground<'run, F: Files, S: Store> {
    run: &'run Run,
    files: F,
    located: Located,
    store: &'run S,
}

impl<'run, F: Files, S: Store> Ground<'run, F, S> {
    pub fn new(run: &'run Run, files: F, located: Located, store: &'run S) -> Ground<'run, F, S> {
        Ground { run, files, located, store }
    }

    pub fn store(&self) -> &'run S {
        self.store
    }

    pub fn transcripts(self) -> Port<'run, ClaudeCodeDir<F>> {
        self.run.port(self.located.transcripts(self.files))
    }

    pub fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.files.read(&self.located.within(path))
    }

    pub fn tail(&self, path: &Path, kept: u64) -> Result<Bytes, FileFailure> {
        let size = self.files.size(path)?.into_inner();
        let start = match i64::try_from(size.saturating_sub(kept)).map(Offset::try_new) {
            Ok(Ok(offset)) => offset,
            Ok(Err(_negative)) | Err(_huge) => Offset::default(),
        };
        self.files.read_from(path, start)
    }

    pub fn located(&self) -> &Located {
        &self.located
    }

    pub fn configured(&self, project: Project) -> Result<Kept, Failure> {
        Ok(self.located.configured(project).kept(&self.files, self.located.repo())?)
    }
}
