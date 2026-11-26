use file_ports::Files;
use port_core::{Port, Run};
use store_ports::Store;
use transcript_ports::ClaudeCodeDir;

use super::projects::Located;

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

    pub fn files(&self) -> &F {
        &self.files
    }

    pub fn transcripts(self) -> Port<'run, ClaudeCodeDir<F>> {
        self.run.port(self.located.transcripts(self.files))
    }
}
