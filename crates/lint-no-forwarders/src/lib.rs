mod forwarders;
mod workspace_types;

use forwarders::Forwarders;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_sweep::Judgement;
use workspace_types::{TypeGathering, WorkspaceTypes};

use lint_core::SourceFile;

#[derive(Debug)]
pub struct NoForwarders;

impl ContextLint for NoForwarders {
    const HEADLINE: Headline = Headline::new("methods do their own work, they do not forward to another type of ours");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = WorkspaceTypes;
    type Context<'head> = WorkspaceTypes;

    type Gathering<'head> = TypeGathering;

    fn gathering(parsed: Parsed<'_, '_>) -> TypeGathering {
        TypeGathering::from(parsed.file())
    }

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        types: &'run WorkspaceTypes,
    ) -> Judgement<'file, 'run> {
        Judgement::watching(Box::new(Forwarders::new(file.head().path(), types)))
    }
}
