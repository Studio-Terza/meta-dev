use lint_core::SourceFile;
use lint_layout::Failure;

use super::super::opened::Opened;

#[derive(Debug)]
pub struct Shelved {
    parsed: Option<Result<SourceFile, Failure>>,
}

impl Shelved {
    pub fn take_parsed(&mut self) -> Option<Result<SourceFile, Failure>> {
        self.parsed.take()
    }

    pub fn parsed(&self) -> Option<&SourceFile> {
        self.parsed.as_ref()?.as_ref().ok()
    }
}

impl From<Result<Opened, Failure>> for Shelved {
    fn from(opened: Result<Opened, Failure>) -> Shelved {
        Shelved { parsed: Some(opened.and_then(SourceFile::try_from)) }
    }
}
