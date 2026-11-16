mod calls;

use graph::Model;
use lint_core::{Headline, ModelLint, Report};

use calls::CallGraph;

#[derive(Debug)]
pub struct NodeRecursion;

impl ModelLint for NodeRecursion {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "syntax is walked once, by the sweep: outside the syntax crate, no function that takes a syntax node is recursive in the call graph; read the node from a sweep hook, a shared fact, or a reader in the syntax crate",
            ),
            CallGraph::of_model(model).recursive_over_nodes(),
        )
    }
}
