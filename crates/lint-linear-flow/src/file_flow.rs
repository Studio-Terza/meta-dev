mod collector;

use std::iter;

use lint_core::{SourceFile, Violation};
use lint_placement::CrateName;

use crate::finding::Rule;
use crate::function_flow::FunctionFlow;
use collector::Collector;

#[derive(Debug)]
pub struct FileFlow<'file> {
    file: &'file SourceFile,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> FileFlow<'file> {
    pub fn of_file(file: &'file SourceFile) -> Vec<FileFlow<'file>> {
        iter::once(file)
            .chain(file.test_modules())
            .map(|module| FileFlow { file: module, functions: Collector::collected(module) })
            .collect()
    }

    pub fn crate_name(&self) -> CrateName {
        CrateName::from(self.file.head().placement().package().root())
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
