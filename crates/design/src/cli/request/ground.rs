use std::path::Path;

use design_model::Project;
use file_ports::{Files, Text};
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

    pub fn read(&self, path: &Path) -> Result<Text, Failure> {
        self.files.read_text(&self.located.within(path)).map_err(Failure::Directives)
    }

    pub fn configured(&self, project: Project) -> Result<Kept, Failure> {
        Ok(self.located.configured(project).kept(&self.files, self.located.repo())?)
    }
}
