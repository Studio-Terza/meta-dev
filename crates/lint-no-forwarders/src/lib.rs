use std::path::Path;
mod forwarders;
mod workspace_types;

use forwarders::{Forwarded, Forwarders};
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, WatchingLint};
use workspace_types::{TypeGathering, WorkspaceTypes};

use lint_core::{SourceFile, Violation};

#[derive(Debug)]
pub struct NoForwarders;

impl ContextLint for NoForwarders {
    const HEADLINE: Headline = Headline::new("methods do their own work, they do not forward to another type of ours");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = WorkspaceTypes;
    type Context<'head> = WorkspaceTypes;

    type Gathering<'head> = TypeGathering;

    fn gathering(parsed: Parsed<'_, '_>) -> TypeGathering {
        TypeGathering::from(<&SourceFile>::from(parsed))
    }
}

impl WatchingLint for NoForwarders {
    type Found<'file> = Forwarded<'file>;
    type Watched<'file> = Forwarders<'file>;

    fn watched(file: &SourceFile) -> Forwarders<'_> {
        Forwarders::from(AsRef::<Path>::as_ref(file.head()))
    }

    fn judged<'file, 'head: 'file>(
        _: &'file SourceFile,
        found: Forwarded<'file>,
        types: &WorkspaceTypes,
    ) -> Vec<Violation<'file>> {
        found.declared(types)
    }
}
