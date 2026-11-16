use std::fmt;

use graph::Model;
use lint_core::{FileHead, Parsed, Report};
use workspace_metadata::Metadata;

pub trait WorkspaceLint {
    type Facts<'head>: Send + Sync + fmt::Debug;

    fn gathered<'head>(parsed: &[Parsed<'_, 'head>]) -> Self::Facts<'head>;

    fn report<'file, 'head: 'file>(
        heads: &'file [FileHead],
        workspaces: &'file [Metadata],
        facts: &'file Self::Facts<'head>,
        model: &'file Model,
    ) -> Report<'file>;
}
