use std::mem;
use std::path::Path;
use std::sync::Mutex;

use crate::repo_file::RepoFile;
use crate::source_file::SourceFile;
use invariants::Unpoisoned;
use lint_layout::Failure;

#[derive(Debug)]
pub struct Unwalked(());

#[derive(Debug, Default)]
pub struct Readout {
    files: Vec<SourceFile>,
    unwalked: Mutex<Vec<Unwalked>>,
    repo_files: Vec<RepoFile>,
    failures: Vec<Failure>,
}

impl Readout {
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    #[must_use]
    pub fn repo_files(&self) -> &[RepoFile] {
        &self.repo_files
    }

    #[must_use]
    pub fn failures(&self) -> &[Failure] {
        &self.failures
    }

    #[must_use]
    pub fn unwalked(&self) -> Vec<Unwalked> {
        mem::take(&mut *self.unwalked.lock().unpoisoned())
    }

    #[must_use]
    pub fn with_file(mut self, file: SourceFile) -> Readout {
        self.files.push(file);
        self.unwalked.get_mut().unpoisoned().push(Unwalked(()));
        self
    }

    #[must_use]
    pub fn with_repo_files(self, found: Vec<RepoFile>) -> Readout {
        let mut repo_files = self.repo_files;
        repo_files.extend(found);
        Readout { repo_files, ..self }
    }

    #[must_use]
    pub fn with_failure(self, failure: Failure) -> Readout {
        let mut failures = self.failures;
        failures.push(failure);
        Readout { failures, ..self }
    }

    #[must_use]
    pub fn sorted(mut self) -> Readout {
        self.files.sort_by(|left, right| AsRef::<Path>::as_ref(left.head()).cmp(AsRef::<Path>::as_ref(right.head())));
        self.repo_files.sort();
        self.repo_files.dedup();
        self
    }
}
