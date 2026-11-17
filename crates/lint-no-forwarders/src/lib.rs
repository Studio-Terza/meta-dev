use std::path::Path;
mod forwarders;
mod workspace_types;

use forwarders::{Forwarded, Forwarders, Scope};
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_sweep::Folding;
use workspace_types::{TypeGathering, WorkspaceTypes};

use lint_core::{SourceFile, Violation};

#[derive(Debug)]
pub struct NoForwarders;

impl ContextLint for NoForwarders {
    const HEADLINE: Headline = Headline::new("methods do their own work, they do not forward to another type of ours");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = WorkspaceTypes;
    type Context<'head> = WorkspaceTypes;

    type Gathering<'head> = Folding<TypeGathering>;

    fn gathering(parsed: Parsed<'_, '_>) -> Folding<TypeGathering> {
        Folding::from(TypeGathering::from(<&SourceFile>::from(parsed)))
    }
}

impl WatchingLint for NoForwarders {
    type Found<'file> = Scope;
    type Watched<'file> = Folding<Forwarders>;

    fn watched(_: &SourceFile) -> Folding<Forwarders> {
        Folding::from(Forwarders)
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Scope,
        types: &WorkspaceTypes,
    ) -> Vec<Violation<'file>> {
        Forwarded::new(AsRef::<Path>::as_ref(file.head()), found).declared(types)
    }
}
