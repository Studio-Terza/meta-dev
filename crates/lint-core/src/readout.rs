use std::mem;
use std::sync::Mutex;

use crate::joins::Joins;
use crate::source_file::SourceFile;
use invariants::Unpoisoned;
use lint_layout::Failure;

#[derive(Debug)]
pub struct Unwalked(());

#[derive(Debug, Default)]
pub struct Readout {
    files: Vec<SourceFile>,
    unwalked: Mutex<Vec<Unwalked>>,
    failures: Vec<Failure>,
}

impl Joins for Readout {
    fn combine(self, other: Readout) -> Readout {
        Readout {
            files: self.files.combine(other.files),
            unwalked: Mutex::new(
                self.unwalked.into_inner().unpoisoned().combine(other.unwalked.into_inner().unpoisoned()),
            ),
            failures: self.failures.combine(other.failures),
        }
    }
}

impl FromIterator<SourceFile> for Readout {
    fn from_iter<Files: IntoIterator<Item = SourceFile>>(read: Files) -> Readout {
        let files: Vec<SourceFile> = read.into_iter().collect();
        let unwalked = files.iter().map(|_| Unwalked(())).collect();
        Readout { files, unwalked: Mutex::new(unwalked), ..Readout::default() }
    }
}

impl FromIterator<Failure> for Readout {
    fn from_iter<Failures: IntoIterator<Item = Failure>>(failures: Failures) -> Readout {
        Readout { failures: failures.into_iter().collect(), ..Readout::default() }
    }
}

impl Readout {
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    #[must_use]
    pub fn failures(&self) -> &[Failure] {
        &self.failures
    }

    #[must_use]
    pub fn unwalked(&self) -> Vec<Unwalked> {
        mem::take(&mut *self.unwalked.lock().unpoisoned())
    }
}
