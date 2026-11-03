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

#[derive(Debug)]
pub struct NoForwarders;

impl Lint for NoForwarders {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let types = WorkspaceTypes::from(shipped.as_slice());
        Report::new(
            Headline::new("methods do their own work, they do not forward to another type of ours"),
            shipped
                .iter()
                .flat_map(|file| {
                    Self::forwarders(file, &types).into_iter().map(move |forwarder| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(forwarder.name().span())),
                            Finding::Forwards { name: forwarder.name(), callee: forwarder.callee() },
                        )
                    })
                })
                .collect(),
        )
    }
}

impl NoForwarders {
    fn forwarders<'file>(file: &'file SourceFile, types: &WorkspaceTypes) -> Vec<Forwarder<'file>> {
        let mut scan = Forwarders::new(types);
        scan.visit_file(file.syntax());
        scan.found()
    }
}
