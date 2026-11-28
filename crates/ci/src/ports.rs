use std::path::Path;

use dispatch_ports::Shell;
use file_ports::{Bytes, Files};
use timer_ports::Clock;

use crate::failure::Failure;

#[derive(Debug)]
pub(crate) struct Ports<'run, F, C, S> {
    files: &'run F,
    clock: &'run C,
    shell: &'run S,
}

impl<'run, F: Files, C: Clock, S: Shell> Ports<'run, F, C, S> {
    pub(crate) fn new(files: &'run F, clock: &'run C, shell: &'run S) -> Ports<'run, F, C, S> {
        Ports { files, clock, shell }
    }

    pub(crate) fn clock(&self) -> &C {
        self.clock
    }

    pub(crate) fn shell(&self) -> &S {
        self.shell
    }

    pub(crate) fn read(&self, path: &Path) -> Result<String, Failure> {
        let text =
            self.files.read_text(path).map_err(|source| Failure::Unreadable { path: path.to_path_buf(), source })?;
        Ok(AsRef::<str>::as_ref(&text).to_owned())
    }

    pub(crate) fn write(&self, path: &Path, text: &str) -> Result<(), Failure> {
        let written = self.files.write(path, Bytes::from(text));
        written.map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }

    pub(crate) fn append(&self, path: &Path, text: &str) -> Result<(), Failure> {
        let appended = self.files.append(path, &Bytes::from(text));
        appended.map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }
}
