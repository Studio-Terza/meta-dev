use workspace_metadata::Metadata;

use crate::report::Report;
use crate::source_file::SourceFile;

pub trait Lint<File = SourceFile> {
    fn report(files: &[File]) -> Report<'_>;
}

pub trait WorkspaceLint {
    fn report<'file>(files: &'file [SourceFile], workspaces: &'file [Metadata]) -> Report<'file>;
}
