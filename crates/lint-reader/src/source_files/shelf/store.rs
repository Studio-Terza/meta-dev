use lint_core::SourceFile;
use lint_layout::Failure;

use super::super::opened::Opened;

#[derive(Debug)]
pub struct Shelved {
    opened: Option<Opened>,
    parsed: Option<Result<SourceFile, Failure>>,
}

impl Shelved {
    pub fn take_parsed(&mut self) -> Option<Result<SourceFile, Failure>> {
        self.parsed.take()
    }

    pub fn opened(&self) -> Option<&Opened> {
        self.opened.as_ref()
    }

    pub fn parsed(&self) -> Option<&SourceFile> {
        self.parsed.as_ref()?.as_ref().ok()
    }
}

impl From<Result<Opened, Failure>> for Shelved {
    fn from(opened: Result<Opened, Failure>) -> Self {
        match opened {
            Ok(opened) => Self { parsed: Some(opened.parsed()), opened: Some(opened) },
            Err(failure) => Self { opened: None, parsed: Some(Err(failure)) },
        }
    }
}
