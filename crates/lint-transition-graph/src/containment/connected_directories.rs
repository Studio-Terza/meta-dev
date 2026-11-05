use lint_core::{Headline, Violation};

use super::cluster::Level;
use super::divided::Divided;
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct ConnectedDirectories;

impl ClusterLint for ConnectedDirectories {
    const HEADLINE: Headline = Headline::new(
        "report only, each directory module is one connected piece of the type and trait graph: types and traits are linked by fields, variants, bounds, trait impls, the signatures of their methods and the names their bodies use, and a directory module whose items fall into groups with no link between them is several modules in one",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.splits::<Divided<'_, '_>>(Level::Module)
    }
}
