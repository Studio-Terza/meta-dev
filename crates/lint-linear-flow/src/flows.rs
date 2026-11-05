use lint_core::{Headline, Report, SourceFile};

use crate::file_flow::FileFlow;
use crate::rule::Rule;

pub trait FlowLint {
    const HEADLINE: &'static str;
    const RULE: Rule;
}

#[derive(Debug)]
pub struct Flows<'file> {
    files: Vec<FileFlow<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Flows<'file> {
    fn from(files: &'file [SourceFile]) -> Flows<'file> {
        Flows { files: FileFlow::of_files(files) }
    }
}

impl<'file> Flows<'file> {
    #[must_use]
    pub fn report<L: FlowLint>(&self) -> Report<'file> {
        Report::new(Headline::new(L::HEADLINE), FileFlow::violations(&self.files, L::RULE))
    }
}

impl<'file> AsRef<[FileFlow<'file>]> for Flows<'file> {
    fn as_ref(&self) -> &[FileFlow<'file>] {
        &self.files
    }
}
