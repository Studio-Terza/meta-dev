use std::fmt;

use file_ports::{Bytes, FileFailure, Files};
use grammar::{Cited, Heading, Queue};
use grammar_lines::Lines;
use grammar_spec_line::Attachment;
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

    pub fn queued(&self, line: impl fmt::Display) -> Result<(), FileFailure> {
        self.files.append(self.located.queue(), &Bytes::new(format!("{line}\n").into_bytes()))
    }

    pub fn titled(&self, spec: &Attachment) -> Option<Heading> {
        let written = spec.to_string();
        let cited = Cited::read(&written)?;
        let text = self.files.read_text(&self.located.spec(cited.project())).ignoring(Ignored::Untitled)?;
        let lines = match text.as_ref().parse::<Lines>() {
            Ok(lines) => lines,
            Err(unsplit) => match unsplit {},
        };
        lines.into_iter().filter_map(|line| Heading::read(&line)).find(|heading| heading.detail() == cited.detail())
    }

    pub fn transcripts(self) -> Port<'run, ClaudeCodeDir<F>> {
        self.run.port(self.located.transcripts(self.files))
    }
}
