mod callee;
mod finding;
mod forwarder;
mod forwarders;
mod workspace_types;

use finding::Finding;
use forwarder::Forwarder;
use forwarders::Forwarders;
use lint_core::{Headline, Lint};
use syn::visit::Visit;
use workspace_types::WorkspaceTypes;

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Debug)]
pub struct NoForwarders;

impl Lint for NoForwarders {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let types = WorkspaceTypes::from(shipped.as_slice());
        Report::new(
            Headline::new("methods do their own work, they do not forward to another type of ours"),
            shipped
                .iter()
                .flat_map(|file| {
                    Self::forwarders(file, &types).into_iter().map(move |forwarder| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(forwarder.name().span())),
                            Finding::Forwards {
                                name: Name::from(forwarder.name()),
                                callee: forwarder.callee().clone(),
                            },
                        )
                    })
                })
                .collect(),
        )
    }
}

impl NoForwarders {
    fn forwarders(file: &SourceFile, types: &WorkspaceTypes) -> Vec<Forwarder> {
        let mut sweep = Forwarders::new(types);
        sweep.visit_file(file.syntax());
        sweep.found()
    }
}
