use crate::report::Report;
use crate::source_file::SourceFile;

pub trait Lint {
    fn report(files: &[SourceFile]) -> Report;
}
