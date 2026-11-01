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
        Readout { files: self.files.into_iter().chain([file]).collect(), ..self }
    }

    #[must_use]
    pub fn with_repo_files(self, found: Vec<RepoFile>) -> Readout {
        Readout { repo_files: self.repo_files.into_iter().chain(found).collect(), ..self }
    }

    #[must_use]
    pub fn with_failure(self, failure: Failure) -> Readout {
        Readout { failures: self.failures.into_iter().chain([failure]).collect(), ..self }
    }

    #[must_use]
    pub fn sorted(mut self) -> Readout {
        self.files.sort_by(|left, right| left.path().cmp(right.path()));
        self.repo_files.sort();
        self.repo_files.dedup();
        self
    }
}
