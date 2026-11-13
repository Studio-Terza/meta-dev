use workspace_metadata::Metadata;

use crate::report::Report;
use crate::source_file::{FileHead, Parsed, SourceFile};
use std::fmt;

pub trait Lint<File = SourceFile> {
    fn report(files: &[File]) -> Report<'_>;
}

pub trait WorkspaceLint {
    type Facts<'head>: Send + Sync + fmt::Debug;

    fn gathered<'head>(parsed: &[Parsed<'_, 'head>]) -> Self::Facts<'head>;

    fn report<'file, 'head: 'file>(
        heads: &'file [FileHead],
        workspaces: &'file [Metadata],
        facts: &'file Self::Facts<'head>,
    ) -> Report<'file>;
}
