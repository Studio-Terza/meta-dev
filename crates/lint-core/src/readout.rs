use crate::failure::Failure;
use crate::source_file::SourceFile;

#[derive(Debug, Default)]
pub struct Readout {
    files: Vec<SourceFile>,
    failures: Vec<Failure>,
}

impl Readout {
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    pub fn add_file(&mut self, file: SourceFile) {
        self.files.push(file);
    }

    pub fn add_failure(&mut self, failure: Failure) {
        self.failures.push(failure);
    }

    pub fn sort(&mut self) {
        self.files.sort_by(|left, right| left.path().cmp(right.path()));
    }
}
