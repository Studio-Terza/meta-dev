use lint_core::{SourceFile, Violation};

use crate::file_flow::FileFlow;
use crate::finding::Rule;

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
        Flows { files: files.iter().flat_map(FileFlow::of_file).collect() }
    }
}

impl<'file> Flows<'file> {
    #[must_use]
    pub fn flagged<L: FlowLint>(file: &'file SourceFile) -> Vec<Violation<'file>> {
        FileFlow::violations(&FileFlow::of_file(file), L::RULE)
    }
}

impl<'file> AsRef<[FileFlow<'file>]> for Flows<'file> {
    fn as_ref(&self) -> &[FileFlow<'file>] {
        &self.files
    }
}
