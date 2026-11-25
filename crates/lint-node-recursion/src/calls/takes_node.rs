use graph::{Function, Named, Resolved, Sited, TypeRef};
use lint_layout::{Name, Word};

#[derive(Debug)]
pub(super) struct TakesNode;

impl TakesNode {
    const NODES: Word<'static> = Word::new("syn");

    pub(super) fn function(function: &Function) -> Option<()> {
        let typed = function.signature().inputs().iter().filter_map(|input| input.typed());
        typed.flat_map(TypeRef::uses).find_map(TakesNode::named)
    }

    fn named(named: &Named<Sited>) -> Option<()> {
        let resolved: &Resolved = named.as_ref();
        let nodes = Name::from(TakesNode::NODES);
        let root = resolved.external().and_then(|external| external.path().as_ref().first());
        root.filter(|found| **found == nodes).map(|_| ())
    }
}
