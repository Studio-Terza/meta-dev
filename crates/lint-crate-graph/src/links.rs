use graph::ItemGraph;
use graph::{Model, References};
use lint_core::{Bound, Site, Violation};
use lint_layout::Name;

mod tangle;

use tangle::Tangle;

#[derive(Debug)]
pub(crate) struct Links<'model> {
    model: &'model Model,
    references: &'model References,
}

impl<'model> From<&'model Model> for Links<'model> {
    fn from(model: &'model Model) -> Links<'model> {
        Links { model, references: model.references() }
    }
}

impl<'model> Links<'model> {
    const TANGLE_ITEMS: Bound = Bound::Most(40);
}

impl<'model> From<&Links<'model>> for Vec<Violation<'model>> {
    fn from(links: &Links<'model>) -> Vec<Violation<'model>> {
        ItemGraph::of_references(links.references.targets().clone())
            .tangles()
            .into_iter()
            .filter_map(|members| {
                let root = members.iter().find_map(|member| links.references.root(**member))?;
                let mut names: Vec<Name> = members
                    .into_iter()
                    .filter_map(|member| links.model.declared(*member))
                    .map(|item| AsRef::<Name>::as_ref(item).clone())
                    .collect();
                names.sort();
                let tangle = Tangle::new(names);
                Links::TANGLE_ITEMS
                    .allows(tangle.items())
                    .map_or(Some(Violation::new(Site::of_file(root), tangle)), |_| None)
            })
            .collect()
    }
}
