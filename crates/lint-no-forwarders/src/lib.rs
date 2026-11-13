mod callee;
mod finding;
mod forwarder;
mod forwarders;
mod workspace_types;

pub use finding::Finding;
use forwarder::Forwarder;
use forwarders::Forwarders;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use syn::visit::Visit;
use workspace_types::WorkspaceTypes;

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct NoForwarders;

impl ContextLint for NoForwarders {
    const HEADLINE: Headline = Headline::new("methods do their own work, they do not forward to another type of ours");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = WorkspaceTypes;
    type Context<'head> = WorkspaceTypes;

    fn contributed(parsed: Parsed<'_, '_>) -> WorkspaceTypes {
        parsed.file().shipped().map_or_default(WorkspaceTypes::from)
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, types: &WorkspaceTypes) -> Vec<Violation<'file>> {
        Self::forwarders(file, types)
            .into_iter()
            .map(|forwarder| {
                Violation::new(
                    Site::new(file.head().path(), LineNumber::from(forwarder.name().span())),
                    Finding::Forwards { name: forwarder.name(), callee: forwarder.callee() },
                )
            })
            .collect()
    }
}

impl NoForwarders {
    fn forwarders<'file>(file: &'file SourceFile, types: &WorkspaceTypes) -> Vec<Forwarder<'file>> {
        let mut scan = Forwarders::new(types);
        scan.visit_file(file.syntax());
        scan.found()
    }
}
