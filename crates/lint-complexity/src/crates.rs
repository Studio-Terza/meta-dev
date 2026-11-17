mod distribution;
mod function;

use std::collections::BTreeMap;

use graph::Model;
use lint_core::Site;
use lint_placement::CrateName;

pub(crate) use distribution::Distribution;
use function::Function;

#[derive(Debug)]
pub(crate) struct Crates<'model>(BTreeMap<CrateName, Vec<Function<'model>>>);

impl<'model> Crates<'model> {
    pub(crate) fn grouped(model: &'model Model) -> Crates<'model> {
        let mut functions: Vec<Function<'model>> = model.functions().filter_map(Function::scored).collect();
        functions.sort_by_key(|function| *AsRef::<Site<'model>>::as_ref(function));
        Crates(functions.into_iter().fold(BTreeMap::new(), |mut crates, function| {
            crates.entry(AsRef::<CrateName>::as_ref(&function).clone()).or_insert_with(Vec::new).push(function);
            crates
        }))
    }

    pub(crate) fn listed(&self) -> impl Iterator<Item = Distribution<'model, '_>> {
        self.0.iter().filter_map(|(name, functions)| Distribution::listing(name, functions))
    }
}
