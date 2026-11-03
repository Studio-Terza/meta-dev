use crate::boundary::Violation;
use crate::report::Report;
use crate::source_file::SourceFile;

pub trait Lint<File = SourceFile> {
    fn report(files: &[File]) -> Report;
}

pub trait FileLint {
    fn violations(file: &SourceFile) -> Vec<Violation>;
}
