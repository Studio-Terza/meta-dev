use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Kind};
use grammar::Grammar;
use grammar::detail::{Cited, Heading};
use grammar::lines::Lines;
use grammar::spec_line::Attachment;
use invariants::Ignoring;
use port_core::{Port, Run};
use store_ports::Store;
use toml_ports::Toml;
use transcript_ports::ClaudeCodeDir;

use super::ignored::Ignored;
use super::local::{Local, Rooted};
use super::projects::{Kept, Located};
use super::queue::Queue;
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

    pub fn files(&self) -> &F {
        &self.files
    }

    pub fn prepared(&self, directory: &Path) -> Result<(), Failure> {
        self.files.create_directories(directory).map_err(Failure::Files)
    }

    pub fn kind(&self, path: &Path) -> Kind {
        self.files.kind(path)
    }

    pub fn queue(&self, file: &Rooted) -> Result<Queue, Failure> {
        self.files
            .read_text(file.as_ref())
            .ignoring(Ignored::Unqueued)
            .map_or(Ok(Queue::default()), |text| Toml::read(text.as_ref()).map_err(Failure::Unparsed))
    }

    pub fn queued(&self, file: &Rooted, queue: &Queue) -> Result<(), Failure> {
        let text = Toml::written(queue).map_err(Failure::Unwritten)?;
        self.files.write(file.as_ref(), Bytes::new(text.into_bytes())).map_err(Failure::Unqueued)
    }

    pub fn titled(&self, specs: &Rooted, spec: &Attachment) -> Option<Heading> {
        let written = spec.to_string();
        let cited = Cited::read(&written)?;
        let text = self.files.read_text(&specs.spec(cited.project())).ignoring(Ignored::Untitled)?;
        Lines::from(text.as_ref())
            .into_iter()
            .filter_map(|line| Heading::read(&line))
            .find(|heading| heading.detail() == cited.detail())
    }

    pub fn workspace(&self) -> &Path {
        self.located.workspace()
    }

    pub fn settings(&self) -> PathBuf {
        self.located.settings()
    }

    pub fn local(&self) -> Local {
        let text = self.files.read_text(&self.located.kept(Kept::Local)).ignoring(Ignored::Unconfigured);
        text.map_or_default(|read| Local::written(read.as_ref()))
    }

    pub fn transcripts(&self) -> Port<'run, ClaudeCodeDir<F>>
    where
        F: Clone,
    {
        self.run.port(self.located.transcripts(self.files.clone()))
    }
}
