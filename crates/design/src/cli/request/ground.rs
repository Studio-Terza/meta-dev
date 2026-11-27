use std::fmt;

use file_ports::{Bytes, FileFailure, Files};
use grammar::Queue;
use invariants::Ignoring;
use port_core::{Port, Run};
use store_ports::Store;
use transcript_ports::ClaudeCodeDir;

use super::ignored::Ignored;
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

    pub fn queue(&self) -> Option<Queue> {
        let text = self.files.read_text(self.located.queue()).ignoring(Ignored::Unqueued)?;
        Some(Queue::read(text.as_ref()))
    }

    pub(crate) fn queued(&self, line: impl fmt::Display) -> Result<(), FileFailure> {
        self.files.append(self.located.queue(), &Bytes::new(format!("{line}\n").into_bytes()))
    }

    pub(crate) fn transcripts(self) -> Port<'run, ClaudeCodeDir<F>> {
        self.run.port(self.located.transcripts(self.files))
    }
}
