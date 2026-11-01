use crate::repo_file::RepoFile;
use crate::source_file::SourceFile;
use lint_layout::Failure;

#[derive(Debug, Default)]
pub struct Readout {
    files: Vec<SourceFile>,
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

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    #[must_use]
    pub fn with_file(self, file: SourceFile) -> Readout {
        let mut files = self.files;
        files.push(file);
        Readout { files, ..self }
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
        self.files.sort_by(|left, right| left.path().cmp(right.path()));
        self.repo_files.sort();
        self.repo_files.dedup();
        self
    }
}
