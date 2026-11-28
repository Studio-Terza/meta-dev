mod boundary;

use std::fmt;
use std::path::{Path, PathBuf};
use std::slice;

use dispatch_ports::{Clock, Deadline, Pause, Ran, Shell, Timeout, Wall};
use invariants::Ignoring;

use crate::cli::request::ignored::Ignored;
pub(crate) use boundary::{Prompt, Reply};

#[derive(Debug)]
pub(crate) struct Crew<'ports, C: Clock, G: Shell> {
    clock: &'ports C,
    shell: &'ports G,
    scratch: PathBuf,
}

#[derive(strum::IntoStaticStr, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum Program {
    Git,
    Gh,
    Cargo,
    Design,
    Claude,
}

impl<'ports, C: Clock, G: Shell> Crew<'ports, C, G> {
    pub(crate) fn new(clock: &'ports C, shell: &'ports G, scratch: PathBuf) -> Crew<'ports, C, G> {
        Crew { clock, shell, scratch }
    }

    pub(crate) fn now(&self) -> Wall {
        self.clock.now()
    }

    pub(crate) fn scratch(&self) -> &Path {
        &self.scratch
    }

    pub(crate) fn spoke(
        &self,
        program: Program,
        arguments: impl IntoIterator<Item: fmt::Display>,
        directory: &Path,
        timeout: Timeout,
    ) -> Option<Ran> {
        let call = program.called(arguments, directory.to_path_buf(), Deadline::after(self.clock.now(), timeout));
        self.shell.ran(&call).ignoring(Ignored::Unrun)
    }

    pub(crate) fn paused(&self, pause: Pause) {
        self.clock.paused(pause);
    }

    pub(crate) fn replied(&self, prompt: &Prompt, timeout: Timeout) -> Option<Reply> {
        self.fanned(slice::from_ref(prompt), timeout).into_iter().next().flatten()
    }

    pub(crate) fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Vec<Option<Reply>> {
        let deadline = Deadline::after(self.clock.now(), timeout);
        let calls: Vec<_> = prompts.iter().map(|prompt| prompt.asked(self.scratch.clone(), deadline)).collect();
        self.shell
            .fanned(&calls)
            .into_iter()
            .map(|ran| ran.ignoring(Ignored::Unrun).as_ref().and_then(Ran::succeeded).and_then(Reply::read))
            .collect()
    }
}
