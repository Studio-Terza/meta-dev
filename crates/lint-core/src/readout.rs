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

    #[must_use]
    pub fn with_file(self, file: SourceFile) -> Readout {
        Readout { files: self.files.into_iter().chain([file]).collect(), ..self }
    }

    #[must_use]
    pub fn with_failure(self, failure: Failure) -> Readout {
        Readout { failures: self.failures.into_iter().chain([failure]).collect(), ..self }
    }

    #[must_use]
    pub fn sorted(mut self) -> Readout {
        self.files.sort_by(|left, right| left.path().cmp(right.path()));
        self
    }
}
