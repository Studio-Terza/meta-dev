mod collector;

use lint_core::{SourceFile, Violation};
use lint_placement::CrateName;
use type_facts::{Facts, Gating};

use crate::finding::Rule;
use crate::function_flow::FunctionFlow;
use collector::Collector;

#[derive(Debug)]
pub struct FileFlow<'file> {
    file: &'file SourceFile,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> FileFlow<'file> {
    pub fn of_files(files: &'file [SourceFile]) -> Vec<FileFlow<'file>> {
        let facts = Facts::of_files(files);
        let shipped = files.iter().map(|file| (file, Gating::Shipped));
        let tests = files.iter().flat_map(SourceFile::test_modules).map(|file| (file, Gating::Tested));
        shipped
            .chain(tests)
            .map(|(file, gating)| FileFlow { file, functions: Collector::collected(file, &facts, gating) })
            .collect()
    }

    pub fn crate_name(&self) -> CrateName {
        CrateName::from(self.file.placement().package().root())
    }

    pub fn functions(&self) -> &[FunctionFlow<'file>] {
        &self.functions
    }

    pub fn violations(flows: &[FileFlow<'file>], rule: Rule) -> Vec<Violation<'file>> {
        let mut found: Vec<Violation<'file>> = flows
            .iter()
            .flat_map(|flow| flow.functions.iter())
            .flat_map(|function| function.violations(rule))
            .collect();
        found.sort_by_key(|violation| *violation.site());
        found
    }
}
