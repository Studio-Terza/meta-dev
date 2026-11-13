use code_model::{Model, References};
use lint_core::{Bound, Site, Violation};
use lint_item_graph::ItemGraph;
use lint_layout::Name;

mod tangle;

use tangle::Tangle;

#[derive(Debug)]
pub(crate) struct Links<'model> {
    model: &'model Model,
    references: References<'model>,
}

impl<'model> From<&'model Model> for Links<'model> {
    fn from(model: &'model Model) -> Links<'model> {
        Links { model, references: References::from(model) }
    }
}

impl<'model> Links<'model> {
    const TANGLE_ITEMS: Bound = Bound::Most(40);

    pub(crate) fn violations(&self) -> Vec<Violation<'model>> {
        ItemGraph::of_references(self.references.targets().clone())
            .tangles()
            .into_iter()
            .filter_map(|members| {
                let root = members.iter().find_map(|member| self.references.root(**member))?;
                let mut names: Vec<Name> = members
                    .into_iter()
                    .filter_map(|member| self.model.declared(*member))
                    .map(|item| AsRef::<Name>::as_ref(item).clone())
                    .collect();
                names.sort();
                let tangle = Tangle::new(names);
                Self::TANGLE_ITEMS
                    .allows(tangle.items())
                    .map_or(Some(Violation::new(Site::of_file(root), tangle)), |_| None)
            })
            .collect()
    }
}
