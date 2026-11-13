mod boundary;
mod unformatted;

use graph::{ItemKind, ItemPath, Model, Placed, Quoted, Resolved};
use lint_core::Site;
use lint_layout::{Word, words};

use boundary::Sql;
pub(crate) use unformatted::Unformatted;

#[derive(Debug)]
pub(crate) struct Queries<'model> {
    model: &'model Model,
    named: Vec<ItemPath>,
}

impl<'model> Queries<'model> {
    const PATHS: [[Word<'static>; 2]; 2] = [words!["store_ports", "query"], words!["store_ports_macros", "query"]];

    pub(crate) fn new(model: &'model Model) -> Queries<'model> {
        Queries { model, named: Self::PATHS.iter().map(|path| path.iter().copied().collect()).collect() }
    }

    pub(crate) fn unformatted(&self, placed: Placed<'model, &'model Quoted>) -> Option<Site<'model>> {
        let quoted = placed.entity();
        self.query(quoted.invoked()).and_then(|_| Sql::from(quoted.as_ref()).unformatted()).map(|_| placed.site())
    }

    fn query(&self, invoked: &Resolved) -> Option<&ItemPath> {
        let declared = invoked
            .crate_item()
            .and_then(|item| self.model.declared(item))
            .filter(|item| item.kind() == ItemKind::Macro)
            .and_then(|item| Some(ItemPath::from_iter([item.krate()?.clone(), AsRef::as_ref(item).clone()])));
        let path = declared.or_else(|| invoked.external().map(|external| external.path().clone()))?;
        self.named.iter().find(|named| **named == path)
    }
}
