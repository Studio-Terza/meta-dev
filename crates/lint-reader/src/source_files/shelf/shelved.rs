use lint_core::SourceFile;
use lint_layout::Failure;

use super::super::located::Opened;

#[derive(Debug)]
pub struct Shelved {
    opened: Option<Opened>,
    parsed: Option<Result<SourceFile, Failure>>,
}

impl Shelved {
    pub fn take_parsed(&mut self) -> Option<Result<SourceFile, Failure>> {
        self.parsed.take()
    }

    pub fn parsed(&self) -> Option<&SourceFile> {
        self.parsed.as_ref()?.as_ref().ok()
    }

    pub fn reparsed(&self) -> Option<SourceFile> {
        self.opened.as_ref()?.parsed().ok()
    }
}

impl From<Result<Opened, Failure>> for Shelved {
    fn from(opened: Result<Opened, Failure>) -> Shelved {
        match opened {
            Ok(opened) => Shelved { parsed: Some(opened.parsed()), opened: Some(opened) },
            Err(failure) => Shelved { opened: None, parsed: Some(Err(failure)) },
        }
    }
}
